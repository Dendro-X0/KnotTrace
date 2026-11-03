use network_core::{
    load_suspicious_activity_settings, run_script_risk_scan, save_suspicious_activity_settings,
    SuspiciousActivitySettings, SuspiciousActivitySnapshot,
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

/// Consent must already be stored (enabled + consent_ack). Bounded heuristics only — not AV.
#[tauri::command]
pub fn run_script_risk_scan_now() -> Result<SuspiciousActivitySnapshot, String> {
    let settings = load_suspicious_activity_settings(&data_dir()).map_err(|e| e.to_string())?;
    if !settings.enabled || !settings.consent_ack {
        return Err(
            "Ack consent and enable Script risk before running a scan.".to_string(),
        );
    }
    Ok(run_script_risk_scan())
}
