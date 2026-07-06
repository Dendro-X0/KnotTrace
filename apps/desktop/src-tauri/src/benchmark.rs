use network_core::{
    delete_benchmark_snapshot, list_benchmark_snapshots, save_benchmark_snapshot, BenchmarkSnapshot,
    ExternalSpeedtestNote,
};
use tauri::State;

use crate::state::{data_dir, AppState};

#[tauri::command]
pub fn list_benchmarks() -> Result<Vec<BenchmarkSnapshot>, String> {
    list_benchmark_snapshots(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_benchmark(
    state: State<'_, AppState>,
    label: String,
    download_mbps: Option<f64>,
    upload_mbps: Option<f64>,
    ping_ms: Option<f64>,
    notes: Option<String>,
) -> Result<BenchmarkSnapshot, String> {
    let report = state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .clone()
        .ok_or_else(|| "Run a health check before saving a benchmark snapshot.".to_string())?;

    let external_speedtest = if download_mbps.is_some() || upload_mbps.is_some() || ping_ms.is_some() {
        Some(ExternalSpeedtestNote {
            download_mbps,
            upload_mbps,
            ping_ms,
            source: "speedtest.net".to_string(),
        })
    } else {
        None
    };

    save_benchmark_snapshot(&data_dir(), &report, label, external_speedtest, notes)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_benchmark(id: String) -> Result<(), String> {
    delete_benchmark_snapshot(&data_dir(), &id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_network_diagnosis(
    state: State<'_, AppState>,
) -> Result<Option<network_core::NetworkDiagnosis>, String> {
    Ok(state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .as_ref()
        .and_then(|report| report.diagnosis.clone()))
}
