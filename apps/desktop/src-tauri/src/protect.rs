use network_core::{
    apply_connect_switch, apply_dns_assist, discover_connect_config, evaluate_protect,
    get_assist_state, load_protect_settings, recommend_connect_discovered, recommend_dns_assist,
    should_notify, AutoProtectAction, AutoProtectResult, HealthGrade,
    HealthReport, ProtectSettings, ProtectStatus, TrustLevel,
};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::monitor::perform_check;
use crate::state::data_dir;

const NOTIFY_COOLDOWN: Duration = Duration::from_secs(300);
const AUTO_PROTECT_COOLDOWN: Duration = Duration::from_secs(600);

pub struct ProtectRuntime {
    pub last_grade: Mutex<Option<HealthGrade>>,
    pub last_notification: Mutex<Option<Instant>>,
    pub last_auto_protect: Mutex<Option<Instant>>,
}

impl ProtectRuntime {
    pub fn new() -> Self {
        Self {
            last_grade: Mutex::new(None),
            last_notification: Mutex::new(None),
            last_auto_protect: Mutex::new(None),
        }
    }
}

pub fn handle_protect_status(app: &AppHandle, report: &HealthReport) -> Result<(), String> {
    let settings = load_protect_settings(&data_dir()).map_err(|error| error.to_string())?;
    let previous_grade = app
        .try_state::<ProtectRuntime>()
        .and_then(|runtime| runtime.last_grade.lock().ok().map(|guard| *guard))
        .flatten();

    let status = evaluate_protect(report, previous_grade, &settings);

    if let Some(runtime) = app.try_state::<ProtectRuntime>() {
        if let Ok(mut guard) = runtime.last_grade.lock() {
            *guard = Some(report.score.grade);
        }
    }

    if settings.enabled {
        if let Some((title, body)) =
            should_notify(
                &status,
                previous_grade,
                report.score.grade,
                &settings,
                report.dns_integrity.as_ref(),
            )
        {
            maybe_send_notification(app, &title, &body)?;
        }
    }

    app.emit("protect-status-updated", &status)
        .map_err(|error| error.to_string())?;

    if settings.enabled && (settings.auto_apply_dns || settings.auto_apply_connect) {
        let app = app.clone();
        let report = report.clone();
        let status = status.clone();
        let settings = settings.clone();
        tauri::async_runtime::spawn(async move {
            if let Ok(result) = run_auto_protect(&app, &report, &status, &settings).await {
                if !result.applied.is_empty() || result.skipped_reason.is_some() {
                    let _ = app.emit("auto-protect-result", &result);
                }
            }
        });
    }

    Ok(())
}

async fn run_auto_protect(
    app: &AppHandle,
    report: &HealthReport,
    status: &ProtectStatus,
    settings: &ProtectSettings,
) -> Result<AutoProtectResult, String> {
    if !auto_protect_allowed(app)? {
        return Ok(AutoProtectResult {
            applied: Vec::new(),
            skipped_reason: Some("Auto-protect cooldown active.".to_string()),
        });
    }

    if !should_run_auto_protect(status, report, settings) {
        return Ok(AutoProtectResult {
            applied: Vec::new(),
            skipped_reason: Some("Current network conditions do not require auto-protect.".to_string()),
        });
    }

    let mut applied = Vec::new();

    if settings.auto_apply_dns {
        match try_auto_dns(report).await {
            Ok(message) => applied.push(AutoProtectAction {
                kind: "dns".to_string(),
                message,
                success: true,
            }),
            Err(error) => applied.push(AutoProtectAction {
                kind: "dns".to_string(),
                message: error,
                success: false,
            }),
        }
    }

    if settings.auto_apply_connect {
        match try_auto_connect(report).await {
            Ok(message) => applied.push(AutoProtectAction {
                kind: "connect".to_string(),
                message,
                success: true,
            }),
            Err(error) => applied.push(AutoProtectAction {
                kind: "connect".to_string(),
                message: error,
                success: false,
            }),
        }
    }

    if applied.iter().any(|action| action.success) {
        mark_auto_protect(app)?;
        let _ = perform_check(app, "auto_protect_applied").await;
    }

    Ok(AutoProtectResult {
        applied,
        skipped_reason: None,
    })
}

fn should_run_auto_protect(
    status: &ProtectStatus,
    report: &HealthReport,
    settings: &ProtectSettings,
) -> bool {
    if settings.auto_apply_on_untrusted_only {
        matches!(status.trust_level, TrustLevel::Untrusted)
            || matches!(report.score.grade, HealthGrade::Poor)
    } else {
        matches!(
            status.trust_level,
            TrustLevel::Untrusted | TrustLevel::Caution
        ) || matches!(report.score.grade, HealthGrade::Fair | HealthGrade::Poor)
    }
}

fn auto_protect_allowed(app: &AppHandle) -> Result<bool, String> {
    let runtime = app
        .try_state::<ProtectRuntime>()
        .ok_or_else(|| "protect runtime unavailable".to_string())?;

    let guard = runtime
        .last_auto_protect
        .lock()
        .map_err(|_| "protect runtime lock poisoned".to_string())?;

    Ok(!guard
        .as_ref()
        .is_some_and(|instant| instant.elapsed() < AUTO_PROTECT_COOLDOWN))
}

fn mark_auto_protect(app: &AppHandle) -> Result<(), String> {
    let runtime = app
        .try_state::<ProtectRuntime>()
        .ok_or_else(|| "protect runtime unavailable".to_string())?;

    let mut guard = runtime
        .last_auto_protect
        .lock()
        .map_err(|_| "protect runtime lock poisoned".to_string())?;
    *guard = Some(Instant::now());
    Ok(())
}

async fn try_auto_dns(report: &HealthReport) -> Result<String, String> {
    if get_assist_state(&data_dir())
        .map_err(|error| error.to_string())?
        .active
    {
        return Err("DNS assist is already active.".to_string());
    }

    let recommendation = recommend_dns_assist(&report.environment, Some(&report.probe))
        .await
        .map_err(|error| error.to_string())?;

    let resolver = recommendation
        .recommended
        .as_ref()
        .filter(|_| recommendation.should_apply)
        .map(|candidate| candidate.resolver.clone())
        .ok_or_else(|| "No DNS improvement available to apply automatically.".to_string())?;

    let result = apply_dns_assist(&data_dir(), &report.environment, &resolver)
        .await
        .map_err(|error| error.to_string())?;

    Ok(result.message)
}

async fn try_auto_connect(report: &HealthReport) -> Result<String, String> {
    let recommendation = recommend_connect_discovered(
        &data_dir(),
        &report.environment,
        Some(&report.score),
    )
    .await
    .map_err(|error| error.to_string())?;

    let switch = recommendation
        .recommended_switch
        .filter(|_| recommendation.should_apply)
        .ok_or_else(|| "No proxy switch available to apply automatically.".to_string())?;

    let config = discover_connect_config(&data_dir())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Proxy API is not configured.".to_string())?;

    let result = apply_connect_switch(&config, &switch.group_name, &switch.target_proxy)
        .await
        .map_err(|error| error.to_string())?;

    Ok(result.message)
}

fn maybe_send_notification(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    let runtime = app
        .try_state::<ProtectRuntime>()
        .ok_or_else(|| "protect runtime unavailable".to_string())?;

    let mut last_notification = runtime
        .last_notification
        .lock()
        .map_err(|_| "protect runtime lock poisoned".to_string())?;

    if last_notification
        .as_ref()
        .is_some_and(|instant| instant.elapsed() < NOTIFY_COOLDOWN)
    {
        return Ok(());
    }

    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|error| error.to_string())?;

    *last_notification = Some(Instant::now());
    Ok(())
}

#[tauri::command]
pub fn get_protect_settings() -> Result<ProtectSettings, String> {
    network_core::load_protect_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_protect_settings(settings: ProtectSettings) -> Result<ProtectSettings, String> {
    network_core::save_protect_settings(&data_dir(), &settings)
        .map_err(|error| error.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub fn get_protect_status(
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<ProtectStatus, String> {
    let settings =
        network_core::load_protect_settings(&data_dir()).map_err(|error| error.to_string())?;
    let report = state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .clone()
        .ok_or_else(|| "No health report available yet.".to_string())?;

    Ok(evaluate_protect(&report, None, &settings))
}
