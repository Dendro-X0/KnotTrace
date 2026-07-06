use crate::types::*;
use std::path::Path;
use std::time::{Duration, Instant};
use thiserror::Error;

const DOWNLOAD_URL: &str = "https://speed.cloudflare.com/__down";
const UPLOAD_URL: &str = "https://speed.cloudflare.com/__up";
const MAX_DOWNLOAD_BYTES: u64 = 20_000_000;
const MAX_UPLOAD_BYTES: u64 = 10_000_000;
const MIN_BYTES: u64 = 500_000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Error)]
pub enum ThroughputError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("throughput probe failed: {0}")]
    Probe(String),
    #[error("invalid settings: {0}")]
    Invalid(String),
}

pub fn settings_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("throughput_settings.json")
}

pub fn load_throughput_settings(data_dir: &Path) -> Result<ThroughputSettings, ThroughputError> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(default_throughput_settings());
    }

    let contents = std::fs::read_to_string(path)?;
    let settings: ThroughputSettings = serde_json::from_str(&contents)?;
    Ok(normalize_throughput_settings(&settings)?)
}

pub fn save_throughput_settings(
    data_dir: &Path,
    settings: &ThroughputSettings,
) -> Result<ThroughputSettings, ThroughputError> {
    let normalized = normalize_throughput_settings(settings)?;
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        settings_path(data_dir),
        serde_json::to_string_pretty(&normalized)?,
    )?;
    Ok(normalized)
}

pub fn normalize_throughput_settings(
    settings: &ThroughputSettings,
) -> Result<ThroughputSettings, ThroughputError> {
    if settings.download_bytes < MIN_BYTES || settings.download_bytes > MAX_DOWNLOAD_BYTES {
        return Err(ThroughputError::Invalid(format!(
            "Download size must be between {MIN_BYTES} and {MAX_DOWNLOAD_BYTES} bytes"
        )));
    }
    if settings.upload_bytes < MIN_BYTES || settings.upload_bytes > MAX_UPLOAD_BYTES {
        return Err(ThroughputError::Invalid(format!(
            "Upload size must be between {MIN_BYTES} and {MAX_UPLOAD_BYTES} bytes"
        )));
    }

    Ok(ThroughputSettings {
        download_bytes: settings.download_bytes,
        upload_bytes: settings.upload_bytes,
    })
}

pub fn calculate_mbps(bytes: usize, duration: Duration) -> f64 {
    let seconds = duration.as_secs_f64();
    if seconds <= 0.0 || bytes == 0 {
        return 0.0;
    }
    (bytes as f64 * 8.0) / seconds / 1_000_000.0
}

pub async fn run_throughput_probe(
    settings: &ThroughputSettings,
) -> Result<ThroughputProbeResult, ThroughputError> {
    let settings = normalize_throughput_settings(settings)?;
    let started = Instant::now();

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| ThroughputError::Probe(error.to_string()))?;

    let download = probe_download(&client, settings.download_bytes).await;
    let upload = probe_upload(&client, settings.upload_bytes).await;

    let download_mbps = download.as_ref().ok().map(|(bytes, duration)| calculate_mbps(*bytes, *duration));
    let upload_mbps = upload.as_ref().ok().map(|(bytes, duration)| calculate_mbps(*bytes, *duration));

    let summary = build_throughput_summary(download_mbps, upload_mbps);

    Ok(ThroughputProbeResult {
        download_mbps,
        upload_mbps,
        download_bytes: settings.download_bytes,
        upload_bytes: settings.upload_bytes,
        duration_ms: started.elapsed().as_millis() as u64,
        summary,
    })
}

async fn probe_download(
    client: &reqwest::Client,
    bytes: u64,
) -> Result<(usize, Duration), ThroughputError> {
    let url = format!("{DOWNLOAD_URL}?bytes={bytes}");
    let started = Instant::now();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| ThroughputError::Probe(error.to_string()))?
        .error_for_status()
        .map_err(|error| ThroughputError::Probe(error.to_string()))?;

    let body = response
        .bytes()
        .await
        .map_err(|error| ThroughputError::Probe(error.to_string()))?;

    Ok((body.len(), started.elapsed()))
}

async fn probe_upload(
    client: &reqwest::Client,
    bytes: u64,
) -> Result<(usize, Duration), ThroughputError> {
    let payload = vec![0u8; bytes as usize];
    let started = Instant::now();
    let response = client
        .post(UPLOAD_URL)
        .body(payload)
        .send()
        .await
        .map_err(|error| ThroughputError::Probe(error.to_string()))?
        .error_for_status()
        .map_err(|error| ThroughputError::Probe(error.to_string()))?;

    let _ = response
        .bytes()
        .await
        .map_err(|error| ThroughputError::Probe(error.to_string()))?;

    Ok((bytes as usize, started.elapsed()))
}

fn build_throughput_summary(download_mbps: Option<f64>, upload_mbps: Option<f64>) -> String {
    match (download_mbps, upload_mbps) {
        (Some(down), Some(up)) => format!("Measured {down:.1} Mbps down / {up:.1} Mbps up."),
        (Some(down), None) => format!("Measured {down:.1} Mbps download. Upload probe failed."),
        (None, Some(up)) => format!("Measured {up:.1} Mbps upload. Download probe failed."),
        (None, None) => "Throughput probe failed.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_mbps() {
        let mbps = calculate_mbps(1_250_000, Duration::from_secs(1));
        assert!((mbps - 10.0).abs() < 0.01);
    }

    #[test]
    fn normalizes_settings() {
        let settings = normalize_throughput_settings(&default_throughput_settings()).expect("ok");
        assert_eq!(settings.download_bytes, 5_000_000);
    }

    #[test]
    fn rejects_oversized_download() {
        let error = normalize_throughput_settings(&ThroughputSettings {
            download_bytes: 50_000_000,
            upload_bytes: 2_000_000,
        })
        .expect_err("too large");
        assert!(error.to_string().contains("Download size"));
    }
}
