mod assist;
mod benchmark;
mod connect;
mod dns_integrity;
mod monitor;
mod protect;
mod state;
mod throughput;
mod tray;
mod updater;

use network_core::{HealthReport, HistoryTrendPoint};
use state::{open_store, AppState, MonitorStatus};
use tauri::{Manager, State, WindowEvent};
use monitor::{perform_check, spawn_background_monitor, ENV_POLL_INTERVAL, HEALTH_RECHECK_INTERVAL};

#[tauri::command]
async fn run_check(app: tauri::AppHandle) -> Result<HealthReport, String> {
    perform_check(&app, "manual").await
}

#[tauri::command]
fn get_last_report(state: State<'_, AppState>) -> Result<Option<HealthReport>, String> {
    state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())
        .map(|guard| guard.clone())
}

#[tauri::command]
fn get_history(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<HealthReport>, String> {
    state
        .store
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .recent(limit.unwrap_or(20))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_history_trends(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<HistoryTrendPoint>, String> {
    state
        .store
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .trend_points(limit.unwrap_or(48))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_monitor_status(state: State<'_, AppState>) -> Result<MonitorStatus, String> {
    Ok(MonitorStatus {
        enabled: *state
            .monitor_enabled
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?,
        last_reason: state
            .last_check_reason
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?
            .clone(),
        poll_interval_secs: ENV_POLL_INTERVAL.as_secs(),
        recheck_interval_secs: HEALTH_RECHECK_INTERVAL.as_secs(),
    })
}

#[tauri::command]
fn set_monitor_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state
        .monitor_enabled
        .lock()
        .map_err(|_| "state lock poisoned".to_string())? = enabled;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let store = open_store()?;
            app.manage(AppState {
                last_report: std::sync::Mutex::new(None),
                monitor_enabled: std::sync::Mutex::new(true),
                last_check_reason: std::sync::Mutex::new("startup".to_string()),
                store: std::sync::Mutex::new(store),
            });
            app.manage(protect::ProtectRuntime::new());

            #[cfg(desktop)]
            tray::setup_tray(app)?;

            #[cfg(desktop)]
            if let Some(window) = app.get_webview_window("main") {
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_handle.hide();
                    }
                });
            }

            let handle = app.handle().clone();
            spawn_background_monitor(handle.clone());
            tauri::async_runtime::spawn(async move {
                let _ = perform_check(&handle, "startup").await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_check,
            get_last_report,
            get_history,
            get_history_trends,
            get_monitor_status,
            set_monitor_enabled,
            dns_integrity::get_dns_integrity_settings,
            dns_integrity::set_dns_integrity_settings,
            dns_integrity::get_dns_integrity_status,
            benchmark::list_benchmarks,
            benchmark::save_benchmark,
            benchmark::delete_benchmark,
            benchmark::get_network_diagnosis,
            assist::recommend_dns,
            assist::get_dns_assist_state,
            assist::apply_dns,
            assist::restore_dns,
            connect::get_connect_config,
            connect::set_connect_config,
            connect::discover_connect,
            connect::recommend_connect,
            connect::apply_connect,
            connect::apply_recommended_connect,
            connect::compare_proxy_paths,
            protect::get_protect_settings,
            protect::set_protect_settings,
            protect::get_protect_status,
            protect::list_auto_protect_log,
            throughput::get_throughput_settings,
            throughput::set_throughput_settings,
            throughput::run_throughput_test,
            updater::get_app_info,
            updater::check_for_updates,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
