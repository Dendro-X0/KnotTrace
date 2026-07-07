use network_core::{HealthReport, HistoryStore};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub last_report: Mutex<Option<HealthReport>>,
    pub monitor_enabled: Mutex<bool>,
    pub last_check_reason: Mutex<String>,
    pub store: Mutex<HistoryStore>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorStatus {
    pub enabled: bool,
    pub last_reason: String,
    pub poll_interval_secs: u64,
    pub recheck_interval_secs: u64,
}

pub fn data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    let new_dir = base.join("KnotTrace");
    let legacy_dir = base.join("NetworkCompanion");

    if new_dir.exists() || !legacy_dir.exists() {
        new_dir
    } else {
        legacy_dir
    }
}

pub fn open_store() -> Result<HistoryStore, String> {
    HistoryStore::open(&data_dir().join("history.db")).map_err(|error| error.to_string())
}

pub fn update_after_report(
    app: &AppHandle,
    report: &HealthReport,
    reason: &str,
) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        *state
            .last_report
            .lock()
            .map_err(|_| "state lock poisoned".to_string())? = Some(report.clone());
        *state
            .last_check_reason
            .lock()
            .map_err(|_| "state lock poisoned".to_string())? = reason.to_string();
    }

    app.state::<AppState>()
        .store
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .insert_report(report)
        .map_err(|error| error.to_string())
}
