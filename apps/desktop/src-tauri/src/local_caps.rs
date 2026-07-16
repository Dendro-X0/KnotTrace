use network_core::{
    apply_local_caps_repair, detect_environment,
    get_local_caps_state as load_local_caps_state, restore_local_caps_repair, LocalCapsRepairResult,
    LocalCapsState,
};
use tauri::AppHandle;

use crate::monitor::perform_check;
use crate::state::data_dir;

#[tauri::command]
pub fn get_local_caps_state() -> Result<LocalCapsState, String> {
    load_local_caps_state(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn apply_local_caps(app: AppHandle) -> Result<LocalCapsRepairResult, String> {
    let environment = detect_environment().map_err(|error| error.to_string())?;
    let result =
        apply_local_caps_repair(&data_dir(), &environment).map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "local_caps_applied").await;
    Ok(result)
}

#[tauri::command]
pub async fn restore_local_caps(app: AppHandle) -> Result<String, String> {
    let message = restore_local_caps_repair(&data_dir()).map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "local_caps_restored").await;
    Ok(message)
}
