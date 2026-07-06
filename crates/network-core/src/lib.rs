//! Core library for Network Companion — observe-only health checks.

mod assist;
mod benchmark;
mod clash_api;
mod connect;
mod diagnosis;
mod dns_integrity;
mod dns_platform;
mod env;
mod fingerprint;
mod probe;
mod protect;
mod score;
mod stability;
mod store;
mod tor;
mod throughput;
mod types;

pub use benchmark::{
    delete_benchmark_snapshot, list_benchmark_snapshots, save_benchmark_snapshot, BenchmarkError,
};
pub use diagnosis::diagnose_network;
pub use assist::{
    apply_dns_assist, get_assist_state, recommend_dns_assist, restore_dns_assist, AssistError,
};
pub use connect::{
    apply_connect_switch, compare_proxy_paths, compare_proxy_paths_discovered,
    discover_connect_config, load_connect_config, recommend_connect,
    recommend_connect_discovered, save_connect_config, ConnectError,
    DEFAULT_PROXY_TEST_URL,
};
pub use dns_integrity::{
    default_dns_integrity_settings, evaluate_dns_integrity, load_dns_integrity_settings,
    save_dns_integrity_settings, DnsIntegrityError,
};
pub use env::detect_environment;
pub use fingerprint::environment_fingerprint;
pub use probe::{measure_tcp_latency, probe_dns_resolver, resolve_dns_addresses, run_quick_probe};
pub use stability::run_stability_probes;
pub use tor::{detect_tor_status, is_tor_socks_endpoint};
pub use protect::{
    default_protect_settings, evaluate_protect, load_protect_settings, save_protect_settings,
    should_notify, ProtectError,
};
pub use score::score_health;
pub use store::{HistoryStore, StoreError};
pub use throughput::{
    load_throughput_settings, normalize_throughput_settings, run_throughput_probe,
    save_throughput_settings, ThroughputError,
};
pub use types::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("probe failed: {0}")]
    Probe(String),
    #[error("environment detection failed: {0}")]
    Environment(String),
}

/// Run a full observe-only health check: detect environment, probe, score.
pub async fn run_health_check() -> Result<HealthReport, CoreError> {
    run_health_check_with_settings(None).await
}

/// Run a health check using optional DNS integrity settings.
pub async fn run_health_check_with_settings(
    dns_integrity_settings: Option<&DnsIntegritySettings>,
) -> Result<HealthReport, CoreError> {
    let environment = detect_environment().map_err(CoreError::Environment)?;
    let probe = run_quick_probe(&environment)
        .await
        .map_err(CoreError::Probe)?;
    let score = score_health(&probe);
    let integrity_settings = dns_integrity_settings
        .cloned()
        .unwrap_or_else(default_dns_integrity_settings);
    let dns_integrity = evaluate_dns_integrity(&environment, &integrity_settings)
        .await
        .ok();
    let stability = Some(run_stability_probes().await);

    let mut report = HealthReport {
        timestamp: chrono::Utc::now(),
        environment,
        probe,
        score,
        dns_integrity,
        diagnosis: None,
        stability,
    };
    report.diagnosis = Some(diagnose_network(&report));

    Ok(report)
}
