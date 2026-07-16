use network_core::{
    detect_environment, environment_fingerprint, evaluate_upstream_pool_proof,
    load_dns_integrity_settings, run_health_check_with_settings, CheckProfile, HealthReport,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub const ENV_POLL_INTERVAL: Duration = Duration::from_secs(15);
pub const HEALTH_RECHECK_INTERVAL: Duration = Duration::from_secs(300);

use crate::state::{update_after_report, data_dir, AppState};
use crate::protect::handle_protect_status;

pub async fn perform_check(app: &AppHandle, reason: &str) -> Result<HealthReport, String> {
    let integrity_settings =
        load_dns_integrity_settings(&data_dir()).map_err(|error| error.to_string())?;

    let profile = if reason.starts_with("manual") {
        CheckProfile::Fast
    } else {
        CheckProfile::Full
    };

    match run_health_check_with_settings(Some(&integrity_settings), profile).await {
        Ok(mut report) => {
            enrich_upstream_pool(app, &mut report)?;
            publish_report(app, &report, reason)?;
            Ok(report)
        }
        Err(error) => {
            let message = error.to_string();
            let _ = app.emit("health-check-failed", &message);
            Err(message)
        }
    }
}

fn enrich_upstream_pool(app: &AppHandle, report: &mut HealthReport) -> Result<(), String> {
    let history = app
        .state::<AppState>()
        .store
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .recent(48)
        .map_err(|error| error.to_string())?;

    report.upstream_pool = evaluate_upstream_pool_proof(report, &history);
    report.recommendations = Some(network_core::build_recommendations(report));
    report.diagnosis = Some(network_core::diagnose_network(report));
    Ok(())
}

pub fn publish_report(
    app: &AppHandle,
    report: &HealthReport,
    reason: &str,
) -> Result<(), String> {
    update_after_report(app, report, reason)?;
#[cfg(desktop)]
    crate::tray::update_tray_status(app, report);
    let _ = handle_protect_status(app, report, reason);
    app.emit("health-report-updated", report)
        .map_err(|error| error.to_string())
}

pub fn spawn_background_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_fingerprint: Option<String> = None;
        let mut last_health_check = Instant::now() - HEALTH_RECHECK_INTERVAL;

        loop {
            tokio::time::sleep(ENV_POLL_INTERVAL).await;

            let monitor_enabled = app
                .try_state::<AppState>()
                .and_then(|state| state.monitor_enabled.lock().ok().map(|guard| *guard))
                .unwrap_or(true);

            if !monitor_enabled {
                continue;
            }

            let fingerprint = match detect_environment() {
                Ok(environment) => environment_fingerprint(&environment),
                Err(error) => {
                    tracing::debug!(target = "network_desktop::monitor", "{error}");
                    continue;
                }
            };

            let environment_changed = last_fingerprint.as_ref() != Some(&fingerprint);
            let recheck_due = last_health_check.elapsed() >= HEALTH_RECHECK_INTERVAL;

            if environment_changed || recheck_due {
                last_fingerprint = Some(fingerprint);
                last_health_check = Instant::now();

                let reason = if environment_changed {
                    "network_change"
                } else {
                    "scheduled"
                };

                if let Err(error) = perform_check(&app, reason).await {
                    tracing::warn!(target = "network_desktop::monitor", "{error}");
                }
            }
        }
    });
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
