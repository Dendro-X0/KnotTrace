//! Core library for Network Companion — observe-only health checks.

mod assist;
mod auto_protect_log;
mod benchmark;
mod clash_api;
mod connect;
mod diagnosis;
mod dns_integrity;
mod dns_platform;
mod egress;
mod env;
mod fingerprint;
mod link_facts;
mod link_platform;
mod local_caps;
mod mtu_assist;
mod network_context;
mod probe;
mod process;
mod protect;
mod reachability;
mod recommendations;
mod score;
mod stability;
mod store;
mod tor;
mod throughput;
mod tunnel_compare;
mod types;
mod upstream_pool;

pub use assist::{
    apply_dns_assist, get_assist_state, recommend_dns_assist, restore_dns_assist, AssistError,
};
pub use auto_protect_log::{
    append_auto_protect_log, list_auto_protect_log, AutoProtectLogError,
};
pub use benchmark::{
    delete_benchmark_snapshot, list_benchmark_snapshots, save_benchmark_snapshot, BenchmarkError,
};
pub use diagnosis::diagnose_network;
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
pub use link_facts::{build_link_facts_report, collect_link_facts};
pub use local_caps::{
    apply_local_caps_repair, collect_local_caps, get_local_caps_state, restore_local_caps_repair,
    LocalCapsError,
};
pub use mtu_assist::{
    apply_mtu_assist_repair, collect_mtu_assist, get_mtu_assist_state, restore_mtu_assist_repair,
    MtuAssistError,
};
pub use egress::{egress_unstable, probe_egress, probe_egress_with_options, EgressProbeOptions};
pub use fingerprint::environment_fingerprint;
pub use network_context::{
    assess_network_context, is_untrusted_context, probe_captive_portal,
    probe_captive_portal_with_options, CaptivePortalProbeOptions,
};
pub use probe::{measure_tcp_latency, probe_dns_resolver, resolve_dns_addresses, run_quick_probe};
pub use stability::run_stability_probes;
pub use tor::{detect_tor_status, is_tor_socks_endpoint};
pub use protect::{
    auto_protect_trigger, background_check_warrants_notification, default_protect_settings,
    evaluate_protect, in_quiet_hours, is_automated_check_reason, load_protect_settings,
    notifications_silenced, parse_hhmm_to_minutes, save_protect_settings, should_notify,
    ProtectError,
};
pub use reachability::{
    classify_reachability_error, error_kind_label, probe_proxy_path_report,
    probe_site_reachability, probe_site_reachability_with_options, proxy_verification_failures,
    site_access_degraded, ReachabilityProbeOptions, PROXY_VERIFICATION_DOMAINS,
};
pub use recommendations::build_recommendations;
pub use score::score_health;
pub use store::{HistoryStore, StoreError};
pub use throughput::{
    load_throughput_settings, normalize_throughput_settings, run_throughput_probe,
    save_throughput_settings, ThroughputError,
};
pub use tunnel_compare::{
    build_compare_report, probe_tunnel_path_compare, should_compare_tunnels, TunnelCompareOptions,
};
pub use upstream_pool::{evaluate_upstream_pool_proof, evaluate_upstream_pool_snapshot};
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
    run_health_check_with_settings(None, CheckProfile::Full).await
}

/// Run a health check using optional DNS integrity settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckProfile {
    Fast,
    Full,
}

pub async fn run_health_check_with_settings(
    dns_integrity_settings: Option<&DnsIntegritySettings>,
    profile: CheckProfile,
) -> Result<HealthReport, CoreError> {
    let environment = detect_environment().map_err(CoreError::Environment)?;
    let probe = run_quick_probe(&environment)
        .await
        .map_err(CoreError::Probe)?;
    let score = score_health(&probe);
    let integrity_settings = dns_integrity_settings
        .cloned()
        .unwrap_or_else(default_dns_integrity_settings);
    let (reachability_options, captive_options, egress_options, tunnel_options) = match profile {
        CheckProfile::Fast => (
            ReachabilityProbeOptions {
                timeout: std::time::Duration::from_secs(2),
                max_domains_per_check: 4,
            },
            CaptivePortalProbeOptions {
                timeout: std::time::Duration::from_secs(2),
            },
            EgressProbeOptions {
                timeout: std::time::Duration::from_secs(2),
                max_endpoints_per_path: 2,
            },
            TunnelCompareOptions {
                timeout: std::time::Duration::from_secs(2),
                max_domains: 2,
            },
        ),
        CheckProfile::Full => (
            ReachabilityProbeOptions {
                timeout: std::time::Duration::from_secs(4),
                max_domains_per_check: 6,
            },
            CaptivePortalProbeOptions::default(),
            EgressProbeOptions::default(),
            TunnelCompareOptions::default(),
        ),
    };

    let dns_integrity_task = evaluate_dns_integrity(&environment, &integrity_settings);
    let site_reachability_task = probe_site_reachability_with_options(
        &integrity_settings.verification_domains,
        &environment,
        reachability_options,
    );
    let proxy_path_report_task = probe_proxy_path_report(
        &integrity_settings.verification_domains,
        &environment,
        reachability_options,
    );
    let captive_portal_task = probe_captive_portal_with_options(captive_options);
    let egress_task = probe_egress_with_options(&environment, egress_options);
    let stability_task = run_stability_probes();
    let tunnel_compare_task = probe_tunnel_path_compare(&environment, tunnel_options);

    let (
        dns_integrity_result,
        site_reachability_result,
        proxy_path_report,
        captive_portal,
        egress,
        stability,
        tunnel_compare,
    ) = tokio::join!(
        dns_integrity_task,
        site_reachability_task,
        proxy_path_report_task,
        captive_portal_task,
        egress_task,
        stability_task,
        tunnel_compare_task
    );

    let dns_integrity = dns_integrity_result.ok();
    let site_reachability = Some(site_reachability_result);
    let network_context = Some(assess_network_context(
        &environment,
        &captive_portal,
        dns_integrity.as_ref(),
        site_reachability.as_ref(),
        &probe,
    ));
    let egress = Some(egress);
    let stability = Some(stability);

    let mut report = HealthReport {
        timestamp: chrono::Utc::now(),
        environment: environment.clone(),
        probe,
        score,
        dns_integrity,
        diagnosis: None,
        stability,
        site_reachability,
        egress,
        network_context,
        recommendations: None,
        proxy_path_report,
        link_facts: None,
        local_caps: None,
        mtu_assist: None,
        tunnel_compare,
        upstream_pool: None,
    };

    // Link facts / local caps / MTU assist are sync OS reads; keep off the async probe critical path.
    report.link_facts = Some(crate::link_facts::collect_link_facts(&environment));
    report.local_caps = Some(crate::local_caps::collect_local_caps(&environment, None));
    // Snapshot-only claim; desktop monitor re-evaluates with history for pool recurrence.
    report.upstream_pool = evaluate_upstream_pool_snapshot(&report);
    report.mtu_assist = Some(collect_mtu_assist(&report, None));
    report.recommendations = Some(build_recommendations(&report));
    report.diagnosis = Some(diagnose_network(&report));

    Ok(report)
}
