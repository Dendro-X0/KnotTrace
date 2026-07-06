use network_core::{
    load_throughput_settings, run_throughput_probe, save_throughput_settings, ThroughputProbeResult,
    ThroughputSettings,
};

use crate::state::data_dir;

#[tauri::command]
pub fn get_throughput_settings() -> Result<ThroughputSettings, String> {
    load_throughput_settings(&data_dir()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_throughput_settings(
    download_bytes: u64,
    upload_bytes: u64,
) -> Result<ThroughputSettings, String> {
    save_throughput_settings(
        &data_dir(),
        &ThroughputSettings {
            download_bytes,
            upload_bytes,
        },
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn run_throughput_test() -> Result<ThroughputProbeResult, String> {
    let settings = load_throughput_settings(&data_dir()).map_err(|error| error.to_string())?;
    run_throughput_probe(&settings)
        .await
        .map_err(|error| error.to_string())
}
