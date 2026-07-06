use network_core::{
    default_dns_integrity_settings, detect_environment, evaluate_dns_integrity,
    load_dns_integrity_settings, save_dns_integrity_settings, DnsIntegritySettings, DnsIntegrityStatus,
};

use crate::state::data_dir;

#[tauri::command]
pub fn get_dns_integrity_settings() -> Result<DnsIntegritySettings, String> {
    load_dns_integrity_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_dns_integrity_settings(
    verification_domains: Vec<String>,
) -> Result<DnsIntegritySettings, String> {
    save_dns_integrity_settings(
        &data_dir(),
        &DnsIntegritySettings { verification_domains },
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_dns_integrity_status() -> Result<DnsIntegrityStatus, String> {
    let settings = load_dns_integrity_settings(&data_dir())
        .unwrap_or_else(|_| default_dns_integrity_settings());
    let environment = detect_environment().map_err(|error| error.to_string())?;
    evaluate_dns_integrity(&environment, &settings)
        .await
        .map_err(|error| error.to_string())
}
