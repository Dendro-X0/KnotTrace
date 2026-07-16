use network_core::{
    apply_mtu_assist_repair, get_mtu_assist_state as load_mtu_assist_state,
    restore_mtu_assist_repair, MtuAssistRepairResult, MtuAssistState,
};
use tauri::{AppHandle, Manager};

use crate::monitor::perform_check;
use crate::state::{data_dir, AppState};

#[tauri::command]
pub fn get_mtu_assist_state() -> Result<MtuAssistState, String> {
    load_mtu_assist_state(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn apply_mtu_assist(app: AppHandle) -> Result<MtuAssistRepairResult, String> {
    let report = {
        let state = app.state::<AppState>();
        let guard = state
            .last_report
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        guard.clone()
    };
    let report = match report {
        Some(report) => report,
        None => perform_check(&app, "mtu_assist_precheck").await?,
    };

    let result =
        apply_mtu_assist_repair(&data_dir(), &report).map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "mtu_assist_applied").await;
    Ok(result)
}

#[tauri::command]
pub async fn restore_mtu_assist(app: AppHandle) -> Result<String, String> {
    let message = restore_mtu_assist_repair(&data_dir()).map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "mtu_assist_restored").await;
    Ok(message)
}
