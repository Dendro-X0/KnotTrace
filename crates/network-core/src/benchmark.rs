use crate::fingerprint::environment_fingerprint;
use crate::types::*;
use std::path::Path;
use thiserror::Error;

const MAX_SNAPSHOTS: usize = 20;

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("benchmark error: {0}")]
    Invalid(String),
}

pub fn snapshots_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("benchmark_snapshots.json")
}

pub fn list_benchmark_snapshots(data_dir: &Path) -> Result<Vec<BenchmarkSnapshot>, BenchmarkError> {
    let path = snapshots_path(data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save_benchmark_snapshot(
    data_dir: &Path,
    report: &HealthReport,
    label: String,
    external_speedtest: Option<ExternalSpeedtestNote>,
    notes: Option<String>,
) -> Result<BenchmarkSnapshot, BenchmarkError> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(BenchmarkError::Invalid(
            "Snapshot label cannot be empty.".to_string(),
        ));
    }

    let diagnosis = report
        .diagnosis
        .as_ref()
        .map(|value| value.clone())
        .unwrap_or_else(|| crate::diagnosis::diagnose_network(report));

    let snapshot = BenchmarkSnapshot {
        id: chrono::Utc::now().timestamp_millis().to_string(),
        timestamp: report.timestamp,
        label: trimmed.to_string(),
        environment_fingerprint: environment_fingerprint(&report.environment),
        health_score: report.score.score,
        health_grade: report.score.grade,
        probe_summary: probe_summary_from_report(report),
        dns_integrity_state: report.dns_integrity.as_ref().map(|status| status.state),
        primary_bottleneck: diagnosis.primary_bottleneck,
        external_speedtest,
        notes: notes.filter(|value| !value.trim().is_empty()),
    };

    let mut snapshots = list_benchmark_snapshots(data_dir)?;
    snapshots.insert(0, snapshot.clone());
    snapshots.truncate(MAX_SNAPSHOTS);

    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        snapshots_path(data_dir),
        serde_json::to_string_pretty(&snapshots)?,
    )?;

    Ok(snapshot)
}

pub fn delete_benchmark_snapshot(data_dir: &Path, id: &str) -> Result<(), BenchmarkError> {
    let mut snapshots = list_benchmark_snapshots(data_dir)?;
    let original_len = snapshots.len();
    snapshots.retain(|snapshot| snapshot.id != id);

    if snapshots.len() == original_len {
        return Err(BenchmarkError::Invalid(format!(
            "Snapshot not found: {id}"
        )));
    }

    std::fs::write(
        snapshots_path(data_dir),
        serde_json::to_string_pretty(&snapshots)?,
    )?;

    Ok(())
}

fn probe_summary_from_report(report: &HealthReport) -> BenchmarkProbeSummary {
    BenchmarkProbeSummary {
        internet_latency_ms: report.probe.internet.as_ref().map(|sample| sample.avg_ms),
        internet_loss_pct: report.probe.internet.as_ref().map(|sample| sample.loss_pct),
        gateway_latency_ms: report.probe.gateway.as_ref().map(|sample| sample.avg_ms),
        dns_latency_ms: report
            .probe
            .dns
            .iter()
            .filter(|sample| sample.success)
            .map(|sample| sample.latency_ms)
            .min_by(|left, right| {
                left.partial_cmp(right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::env;

    fn sample_report() -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "host".to_string(),
                interfaces: Vec::new(),
                dns_servers: Vec::new(),
                proxy: ProxySettings {
                    enabled: false,
                    server: None,
                    source: "none".to_string(),
                },
                tags: Vec::new(),
                default_gateway: None,
                active_interface: None,
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: Some(LatencySample {
                    target: "internet".to_string(),
                    avg_ms: 25.0,
                    loss_pct: 0.0,
                }),
                dns: Vec::new(),
                duration_ms: 0,
            },
            score: HealthScore {
                grade: HealthGrade::Good,
                score: 90,
                summary: "ok".to_string(),
                reasons: Vec::new(),
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
        }
    }

    #[test]
    fn saves_and_lists_snapshots() {
        let dir = env::temp_dir().join(format!(
            "network-companion-benchmark-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let snapshot = save_benchmark_snapshot(&dir, &sample_report(), "baseline".to_string(), None, None)
            .expect("save");

        let listed = list_benchmark_snapshots(&dir).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, snapshot.id);
        assert_eq!(listed[0].label, "baseline");

        delete_benchmark_snapshot(&dir, &snapshot.id).expect("delete");
        assert!(list_benchmark_snapshots(&dir).unwrap().is_empty());
    }
}
