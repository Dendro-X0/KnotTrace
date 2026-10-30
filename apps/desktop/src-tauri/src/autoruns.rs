use network_core::{load_autorun_settings, save_autorun_settings, AutorunSettings};
use serde::Deserialize;

use crate::state::data_dir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutorunSettingsUpdate {
    pub enabled: bool,
    pub consent_ack: bool,
}

#[tauri::command]
pub fn get_autorun_settings() -> Result<AutorunSettings, String> {
    load_autorun_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_autorun_settings(update: AutorunSettingsUpdate) -> Result<AutorunSettings, String> {
    let settings = AutorunSettings {
        enabled: update.enabled,
        consent_ack: update.consent_ack,
    };
    save_autorun_settings(&data_dir(), &settings).map_err(|error| error.to_string())
}
