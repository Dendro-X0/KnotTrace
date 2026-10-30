use network_core::{
    load_process_egress_settings, save_process_egress_settings, ProcessEgressSettings,
};
use serde::Deserialize;

use crate::state::data_dir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEgressSettingsUpdate {
    pub enabled: bool,
    pub consent_ack: bool,
}

#[tauri::command]
pub fn get_process_egress_settings() -> Result<ProcessEgressSettings, String> {
    load_process_egress_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_process_egress_settings(
    update: ProcessEgressSettingsUpdate,
) -> Result<ProcessEgressSettings, String> {
    let settings = ProcessEgressSettings {
        enabled: update.enabled,
        consent_ack: update.consent_ack,
    };
    save_process_egress_settings(&data_dir(), &settings).map_err(|error| error.to_string())
}
