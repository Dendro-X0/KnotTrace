use network_core::{
    apply_connect_switch, compare_proxy_paths_discovered, detect_environment,
    discover_connect_config, load_connect_config, recommend_connect_discovered,
    save_connect_config, ConnectApplyResult, ConnectConfig, ConnectRecommendation,
    ProxyPathComparison,
};
use tauri::State;

use crate::monitor::perform_check;
use crate::state::{data_dir, AppState};

#[tauri::command]
pub async fn get_connect_config() -> Result<Option<ConnectConfig>, String> {
    load_connect_config(&data_dir())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_connect_config(
    api_base: String,
    secret: Option<String>,
) -> Result<ConnectConfig, String> {
    let config = ConnectConfig {
        api_base,
        secret: secret.filter(|value| !value.trim().is_empty()),
        auto_discovered: false,
    };
    save_connect_config(&data_dir(), &config).map_err(|error| error.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn discover_connect() -> Result<Option<ConnectConfig>, String> {
    discover_connect_config(&data_dir())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn recommend_connect(state: State<'_, AppState>) -> Result<ConnectRecommendation, String> {
    let environment = detect_environment().map_err(|error| error.to_string())?;
    let health = state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .as_ref()
        .map(|report| report.score.clone());

    recommend_connect_discovered(&data_dir(), &environment, health.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn apply_connect(
    app: tauri::AppHandle,
    group_name: String,
    target_proxy: String,
) -> Result<ConnectApplyResult, String> {
    let config = discover_connect_config(&data_dir())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| {
            "No proxy API configured. Enable external-controller in Mihomo or sing-box.".to_string()
        })?;

    let result = apply_connect_switch(&config, &group_name, &target_proxy)
        .await
        .map_err(|error| error.to_string())?;
    let _ = perform_check(&app, "connect_switch_applied").await;
    Ok(result)
}

#[tauri::command]
pub async fn apply_recommended_connect(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ConnectApplyResult, String> {
    let recommendation = recommend_connect(state).await?;
    let switch = recommendation
        .recommended_switch
        .ok_or_else(|| "No proxy switch is recommended right now.".to_string())?;

    apply_connect(app, switch.group_name, switch.target_proxy).await
}

#[tauri::command]
pub async fn compare_proxy_paths(group_name: String) -> Result<ProxyPathComparison, String> {
    compare_proxy_paths_discovered(&data_dir(), &group_name)
        .await
        .map_err(|error| error.to_string())
}
