use network_core::{
    detect_environment, run_local_repair_action, LocalRepairActionKind, LocalRepairResult,
};
use tauri::AppHandle;

use crate::monitor::perform_check;

#[tauri::command]
pub async fn run_local_repair(
    app: AppHandle,
    action: LocalRepairActionKind,
) -> Result<LocalRepairResult, String> {
    let environment = detect_environment().map_err(|error| error.to_string())?;
    let result = run_local_repair_action(action, &environment).map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "local_repair_applied").await;
    Ok(result)
}
