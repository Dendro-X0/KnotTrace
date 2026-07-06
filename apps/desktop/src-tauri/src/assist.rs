use network_core::{
    apply_dns_assist, detect_environment, get_assist_state, recommend_dns_assist, restore_dns_assist,
    DnsApplyResult, DnsAssistRecommendation, DnsAssistState,
};
use tauri::{AppHandle, State};

use crate::monitor::perform_check;
use crate::state::{data_dir, AppState};

#[tauri::command]
pub async fn recommend_dns(state: State<'_, AppState>) -> Result<DnsAssistRecommendation, String> {
    let environment = detect_environment().map_err(|error| error.to_string())?;
    let probe = state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .as_ref()
        .map(|report| report.probe.clone());

    recommend_dns_assist(&environment, probe.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_dns_assist_state() -> Result<DnsAssistState, String> {
    get_assist_state(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn apply_dns(
    app: AppHandle,
    resolver: String,
) -> Result<DnsApplyResult, String> {
    let environment = detect_environment().map_err(|error| error.to_string())?;
    let result = apply_dns_assist(&data_dir(), &environment, &resolver)
        .await
        .map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "dns_assist_applied").await;
    Ok(result)
}

#[tauri::command]
pub async fn restore_dns(app: AppHandle) -> Result<String, String> {
    let message = restore_dns_assist(&data_dir()).map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "dns_assist_restored").await;
    Ok(message)
}
