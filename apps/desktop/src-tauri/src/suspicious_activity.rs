use network_core::{
    load_suspicious_activity_settings, save_suspicious_activity_settings, SuspiciousActivitySettings,
};
use serde::Deserialize;

use crate::state::data_dir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuspiciousActivitySettingsUpdate {
    pub enabled: bool,
    pub consent_ack: bool,
}

#[tauri::command]
pub fn get_suspicious_activity_settings() -> Result<SuspiciousActivitySettings, String> {
    load_suspicious_activity_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_suspicious_activity_settings(
    update: SuspiciousActivitySettingsUpdate,
) -> Result<SuspiciousActivitySettings, String> {
    let settings = SuspiciousActivitySettings {
        enabled: update.enabled,
        consent_ack: update.consent_ack,
    };
    save_suspicious_activity_settings(&data_dir(), &settings).map_err(|error| error.to_string())
}
