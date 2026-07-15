use chrono::Utc;
use network_core::{
    append_auto_protect_log, evaluate_protect, load_protect_settings,
    AutoProtectAction, AutoProtectLogEntry, AutoProtectResult, HealthGrade, HealthReport,
    ProtectSettings, ProtectStatus, TrustLevel,
};
use network_core::{
    apply_connect_switch, apply_dns_assist, auto_protect_trigger,
    background_check_warrants_notification, discover_connect_config, get_assist_state,
    is_automated_check_reason, recommend_connect_discovered, recommend_dns_assist, should_notify,
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
    pub pending_digest: Mutex<Vec<(String, String)>>,
}

impl ProtectRuntime {
    pub fn new() -> Self {
        Self {
            last_grade: Mutex::new(None),
            last_notification: Mutex::new(None),
            last_auto_protect: Mutex::new(None),
            pending_digest: Mutex::new(Vec::new()),
        }
    }
}

pub fn handle_protect_status(
    app: &AppHandle,
    report: &HealthReport,
    check_reason: &str,
) -> Result<(), String> {
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
        if let Some((title, body)) = should_notify(
            &status,
            previous_grade,
            report.score.grade,
            &settings,
            report.dns_integrity.as_ref(),
        ) {
            let should_show = !is_automated_check_reason(check_reason)
                || background_check_warrants_notification(&status, report, previous_grade);
            if should_show {
                maybe_send_notification(app, &settings, &title, &body)?;
            }
        }
    }

    app.emit("protect-status-updated", &status)
        .map_err(|error| error.to_string())?;

    if settings.enabled && (settings.auto_apply_dns || settings.auto_apply_connect) {
        let app = app.clone();
        let report = report.clone();
        let status = status.clone();
        let settings = settings.clone();
        let check_reason = check_reason.to_string();
        tauri::async_runtime::spawn(async move {
            if let Ok(result) =
                run_auto_protect(&app, &report, &status, &settings, &check_reason).await
            {
                if !result.applied.is_empty() || result.skipped_reason.is_some() {
                    let _ = app.emit("auto-protect-result", &result);
                }
            }
        });
    }

    Ok(())
}

pub fn toggle_do_not_disturb(app: &AppHandle) -> Result<ProtectSettings, String> {
    let mut settings =
        load_protect_settings(&data_dir()).map_err(|error| error.to_string())?;
    settings.do_not_disturb = !settings.do_not_disturb;
    save_and_publish_protect_settings(app, &settings)
}

pub fn save_and_publish_protect_settings(
    app: &AppHandle,
    settings: &ProtectSettings,
) -> Result<ProtectSettings, String> {
    network_core::save_protect_settings(&data_dir(), settings)
        .map_err(|error| error.to_string())?;

    #[cfg(desktop)]
    crate::tray::sync_tray_do_not_disturb(app, settings.do_not_disturb);

    if let Some(state) = app.try_state::<crate::state::AppState>() {
        let report = state
            .last_report
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?
            .clone();

        if let Some(report) = report {
            let previous_grade = app
                .try_state::<ProtectRuntime>()
                .and_then(|runtime| runtime.last_grade.lock().ok().map(|guard| *guard))
                .flatten();
            let status = evaluate_protect(&report, previous_grade, settings);
            let _ = app.emit("protect-status-updated", &status);
        }
    }

    Ok(settings.clone())
}

async fn run_auto_protect(
    app: &AppHandle,
    report: &HealthReport,
    status: &ProtectStatus,
    settings: &ProtectSettings,
    check_reason: &str,
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

    let trigger = auto_protect_trigger(status, report, settings);
    let mut applied = Vec::new();

    if settings.auto_apply_dns {
        match try_auto_dns(report, settings).await {
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
        match try_auto_connect(report, settings).await {
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

    for action in &applied {
        let entry = AutoProtectLogEntry {
            timestamp: Utc::now(),
            kind: action.kind.clone(),
            success: action.success,
            message: action.message.clone(),
            trigger: trigger.clone(),
            check_reason: check_reason.to_string(),
            rollback_hint: rollback_hint_for_kind(&action.kind),
        };
        let _ = append_auto_protect_log(&data_dir(), &entry).map_err(|error| error.to_string());
    }

    if !applied.is_empty() {
        let _ = app.emit("auto-protect-log-updated", ());
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

fn rollback_hint_for_kind(kind: &str) -> String {
    match kind {
        "dns" => "Restore DNS Assist from the DNS page.".to_string(),
        "connect" => "Switch proxy node from the Connect page.".to_string(),
        _ => "Review the related Assist page.".to_string(),
    }
}

fn should_run_auto_protect(
    status: &ProtectStatus,
    report: &HealthReport,
    settings: &ProtectSettings,
) -> bool {
    if settings.auto_recover_dns_integrity
        && settings.auto_apply_dns
        && dns_integrity_requires_recovery(report)
    {
        return true;
    }

    if settings.auto_recover_site_access
        && settings.auto_apply_connect
        && site_access_requires_recovery(report)
    {
        return true;
    }

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

fn dns_integrity_requires_recovery(report: &HealthReport) -> bool {
    report.dns_integrity.as_ref().is_some_and(|integrity| {
        matches!(
            integrity.state,
            network_core::DnsIntegrityState::Caution | network_core::DnsIntegrityState::Suspicious
        ) && matches!(
            integrity.confidence,
            network_core::DnsIntegrityConfidence::Medium | network_core::DnsIntegrityConfidence::High
        )
    })
}

fn site_access_requires_recovery(report: &HealthReport) -> bool {
    report
        .site_reachability
        .as_ref()
        .is_some_and(network_core::site_access_degraded)
        && report.environment.proxy.enabled
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

async fn try_auto_dns(report: &HealthReport, settings: &ProtectSettings) -> Result<String, String> {
    if get_assist_state(&data_dir())
        .map_err(|error| error.to_string())?
        .active
    {
        return Err("DNS assist is already active.".to_string());
    }

    let recommendation = recommend_dns_assist(&report.environment, Some(&report.probe))
        .await
        .map_err(|error| error.to_string())?;

    let integrity_recovery =
        settings.auto_recover_dns_integrity && dns_integrity_requires_recovery(report);

    let resolver = recommendation
        .recommended
        .as_ref()
        .filter(|_| recommendation.should_apply || integrity_recovery)
        .map(|candidate| candidate.resolver.clone())
        .ok_or_else(|| {
            if integrity_recovery {
                "DNS integrity recovery could not find a trusted resolver to apply.".to_string()
            } else {
                "No DNS improvement available to apply automatically.".to_string()
            }
        })?;

    let result = apply_dns_assist(&data_dir(), &report.environment, &resolver)
        .await
        .map_err(|error| error.to_string())?;

    Ok(result.message)
}

async fn try_auto_connect(report: &HealthReport, settings: &ProtectSettings) -> Result<String, String> {
    let recommendation = recommend_connect_discovered(
        &data_dir(),
        &report.environment,
        Some(&report.score),
    )
    .await
    .map_err(|error| error.to_string())?;

    let site_recovery =
        settings.auto_recover_site_access && site_access_requires_recovery(report);

    let switch = recommendation
        .recommended_switch
        .filter(|_| recommendation.should_apply || site_recovery)
        .ok_or_else(|| {
            if site_recovery {
                "Site access recovery could not find a better proxy node.".to_string()
            } else {
                "No proxy switch available to apply automatically.".to_string()
            }
        })?;

    let config = discover_connect_config(&data_dir())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Proxy API is not configured.".to_string())?;

    let result = apply_connect_switch(&config, &switch.group_name, &switch.target_proxy)
        .await
        .map_err(|error| error.to_string())?;

    Ok(result.message)
}

fn maybe_send_notification(
    app: &AppHandle,
    settings: &ProtectSettings,
    title: &str,
    body: &str,
) -> Result<(), String> {
    let runtime = app
        .try_state::<ProtectRuntime>()
        .ok_or_else(|| "protect runtime unavailable".to_string())?;

    if settings.notify_digest_only {
        {
            let mut pending = runtime
                .pending_digest
                .lock()
                .map_err(|_| "protect runtime lock poisoned".to_string())?;
            pending.push((title.to_string(), body.to_string()));
        }

        let cooldown_active = {
            let last_notification = runtime
                .last_notification
                .lock()
                .map_err(|_| "protect runtime lock poisoned".to_string())?;
            last_notification
                .as_ref()
                .is_some_and(|instant| instant.elapsed() < NOTIFY_COOLDOWN)
        };

        if cooldown_active {
            return Ok(());
        }

        let digest_items = {
            let mut pending = runtime
                .pending_digest
                .lock()
                .map_err(|_| "protect runtime lock poisoned".to_string())?;
            std::mem::take(&mut *pending)
        };

        if digest_items.is_empty() {
            return Ok(());
        }

        let (digest_title, digest_body) = format_digest_notification(&digest_items);
        app.notification()
            .builder()
            .title(digest_title)
            .body(digest_body)
            .show()
            .map_err(|error| error.to_string())?;

        let mut last_notification = runtime
            .last_notification
            .lock()
            .map_err(|_| "protect runtime lock poisoned".to_string())?;
        *last_notification = Some(Instant::now());
        return Ok(());
    }

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

fn format_digest_notification(items: &[(String, String)]) -> (String, String) {
    let count = items.len();
    let title = if count == 1 {
        items[0].0.clone()
    } else {
        format!("KnotTrace — {count} network updates")
    };

    let mut lines = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (item_title, item_body) in items {
        if seen.insert(item_title.clone()) {
            lines.push(format!("• {item_title}: {item_body}"));
        }
    }

    (title, lines.join("\n"))
}

#[tauri::command]
pub fn get_protect_settings() -> Result<ProtectSettings, String> {
    network_core::load_protect_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_protect_settings(app: tauri::AppHandle, settings: ProtectSettings) -> Result<ProtectSettings, String> {
    save_and_publish_protect_settings(&app, &settings)
}

#[tauri::command]
pub fn list_auto_protect_log(limit: Option<usize>) -> Result<Vec<AutoProtectLogEntry>, String> {
    network_core::list_auto_protect_log(&data_dir(), limit.unwrap_or(20))
        .map_err(|error| error.to_string())
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
