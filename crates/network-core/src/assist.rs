use crate::dns_platform;
use crate::probe::probe_dns_resolver;
use crate::types::*;
use chrono::Utc;
use std::path::{Path, PathBuf};
use thiserror::Error;

const DNS_PROBE_QUERY: &str = "example.com";
const DNS_SLOW_MS: f64 = 80.0;
const DNS_GOOD_MS: f64 = 50.0;
const IMPROVEMENT_MIN_MS: f64 = 15.0;
const IMPROVEMENT_MIN_RATIO: f64 = 0.20;

const PUBLIC_RESOLVERS: &[(&str, &str)] = &[
    ("1.1.1.1", "Cloudflare"),
    ("1.0.0.1", "Cloudflare (alt)"),
    ("8.8.8.8", "Google"),
    ("9.9.9.9", "Quad9"),
];

#[derive(Debug, Error)]
pub enum AssistError {
    #[error("environment detection failed: {0}")]
    Environment(String),
    #[error("dns platform error: {0}")]
    Platform(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("assist unavailable: {0}")]
    Unavailable(String),
}

pub fn backup_path(data_dir: &Path) -> PathBuf {
    data_dir.join("dns_backup.json")
}

pub fn load_backup(data_dir: &Path) -> Result<Option<DnsBackup>, AssistError> {
    let path = backup_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&contents)?))
}

pub fn save_backup(data_dir: &Path, backup: &DnsBackup) -> Result<(), AssistError> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(backup_path(data_dir), serde_json::to_string_pretty(backup)?)?;
    Ok(())
}

pub fn clear_backup(data_dir: &Path) -> Result<(), AssistError> {
    let path = backup_path(data_dir);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn get_assist_state(data_dir: &Path) -> Result<DnsAssistState, AssistError> {
    let backup = load_backup(data_dir)?;
    Ok(DnsAssistState {
        active: backup.is_some(),
        backup,
        can_apply: dns_platform::can_apply(),
        platform_note: dns_platform::platform_note(),
    })
}

pub fn resolve_interface_alias(environment: &EnvironmentSnapshot) -> Option<String> {
    let iface = environment
        .interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| environment.interfaces.iter().find(|iface| iface.is_up))?;

    dns_platform::dns_target_for_interface(iface)
}

pub async fn recommend_dns_assist(
    environment: &EnvironmentSnapshot,
    probe: Option<&ProbeResult>,
) -> Result<DnsAssistRecommendation, AssistError> {
    let current_resolvers = environment
        .dns_servers
        .iter()
        .map(|server| server.address.clone())
        .collect::<Vec<_>>();

    let current_latency_ms = probe.and_then(|result| {
        result
            .dns
            .iter()
            .filter(|sample| sample.success)
            .map(|sample| sample.latency_ms)
            .reduce(f64::min)
    });

    let mut candidates = Vec::new();
    for (resolver, label) in PUBLIC_RESOLVERS {
        let sample = probe_dns_resolver(resolver, DNS_PROBE_QUERY).await;
        candidates.push(DnsCandidateResult {
            resolver: resolver.to_string(),
            label: label.to_string(),
            latency_ms: sample.latency_ms,
            success: sample.success,
        });
    }

    let recommended = candidates
        .iter()
        .filter(|candidate| candidate.success)
        .min_by(|left, right| {
            left.latency_ms
                .partial_cmp(&right.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();

    let (should_apply, reason) = evaluate_recommendation(current_latency_ms, &recommended);

    Ok(DnsAssistRecommendation {
        current_latency_ms,
        current_resolvers,
        candidates,
        recommended,
        should_apply,
        reason,
        interface_alias: resolve_interface_alias(environment),
    })
}

fn evaluate_recommendation(
    current_latency_ms: Option<f64>,
    recommended: &Option<DnsCandidateResult>,
) -> (bool, String) {
    let Some(candidate) = recommended else {
        return (
            false,
            "No working public DNS resolver could be reached.".to_string(),
        );
    };

    let Some(current_ms) = current_latency_ms else {
        return (
            true,
            format!(
                "Current DNS could not be measured. {} looks reachable at {:.0} ms.",
                candidate.label, candidate.latency_ms
            ),
        );
    };

    if current_ms <= DNS_GOOD_MS {
        return (
            false,
            format!(
                "Current DNS is already fast ({current_ms:.0} ms). No change recommended."
            ),
        );
    }

    let improvement_ms = current_ms - candidate.latency_ms;
    let improvement_ratio = improvement_ms / current_ms;

    if current_ms >= DNS_SLOW_MS
        && improvement_ms >= IMPROVEMENT_MIN_MS
        && improvement_ratio >= IMPROVEMENT_MIN_RATIO
    {
        return (
            true,
            format!(
                "{} could reduce DNS latency from {current_ms:.0} ms to {:.0} ms.",
                candidate.label, candidate.latency_ms
            ),
        );
    }

    (
        false,
        format!(
            "Current DNS is {current_ms:.0} ms. {} is {:.0} ms, which is not a large enough improvement to recommend automatically.",
            candidate.label, candidate.latency_ms
        ),
    )
}

pub async fn apply_dns_assist(
    data_dir: &Path,
    environment: &EnvironmentSnapshot,
    resolver: &str,
) -> Result<DnsApplyResult, AssistError> {
    if !dns_platform::can_apply() {
        return Err(AssistError::Unavailable(dns_platform::platform_note()));
    }

    if load_backup(data_dir)?.is_some() {
        return Err(AssistError::Unavailable(
            "DNS assist is already active. Restore the original DNS before applying again."
                .to_string(),
        ));
    }

    let interface_alias = resolve_interface_alias(environment).ok_or_else(|| {
        AssistError::Unavailable("Could not determine the active network interface.".to_string())
    })?;

    let before = probe_dns_resolver("system", DNS_PROBE_QUERY).await;
    let before_latency_ms = if before.success {
        before.latency_ms
    } else {
        recommend_dns_assist(environment, None)
            .await?
            .current_latency_ms
            .unwrap_or(999.0)
    };

    let (previous_servers, use_dhcp) =
        dns_platform::read_dns_config(&interface_alias).map_err(AssistError::Platform)?;

    let backup = DnsBackup {
        interface_alias: interface_alias.clone(),
        previous_servers,
        use_dhcp,
        applied_resolver: resolver.to_string(),
        applied_at: Utc::now(),
    };

    save_backup(data_dir, &backup)?;
    dns_platform::set_static_dns(&interface_alias, resolver).map_err(AssistError::Platform)?;

    tokio::time::sleep(std::time::Duration::from_millis(750)).await;
    let after = probe_dns_resolver("system", DNS_PROBE_QUERY).await;
    let after_latency_ms = if after.success {
        after.latency_ms
    } else {
        f64::MAX
    };

    let kept = after.success
        && (after_latency_ms + 5.0 < before_latency_ms || before_latency_ms >= DNS_SLOW_MS);

    if kept {
        return Ok(DnsApplyResult {
            applied_resolver: resolver.to_string(),
            before_latency_ms,
            after_latency_ms,
            kept: true,
            message: format!(
                "DNS updated to {resolver}. Latency improved from {before_latency_ms:.0} ms to {after_latency_ms:.0} ms."
            ),
        });
    }

    restore_dns_assist(data_dir)?;
    Err(AssistError::Unavailable(format!(
        "Tried {resolver}, but DNS did not improve enough ({before_latency_ms:.0} ms -> {after_latency_ms:.0} ms). Original DNS was restored."
    )))
}

pub fn restore_dns_assist(data_dir: &Path) -> Result<String, AssistError> {
    if !dns_platform::can_apply() {
        return Err(AssistError::Unavailable(dns_platform::platform_note()));
    }

    let backup = load_backup(data_dir)?.ok_or_else(|| {
        AssistError::Unavailable("No DNS backup found. Nothing to restore.".to_string())
    })?;

    dns_platform::restore_dns_config(
        &backup.interface_alias,
        &backup.previous_servers,
        backup.use_dhcp,
    )
    .map_err(AssistError::Platform)?;

    clear_backup(data_dir)?;

    Ok(format!(
        "Restored DNS settings for {}.",
        backup.interface_alias
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommends_when_dns_is_slow_and_candidate_is_faster() {
        let candidate = DnsCandidateResult {
            resolver: "1.1.1.1".to_string(),
            label: "Cloudflare".to_string(),
            latency_ms: 20.0,
            success: true,
        };
        let (should_apply, _) = evaluate_recommendation(Some(120.0), &Some(candidate));
        assert!(should_apply);
    }

    #[test]
    fn skips_recommendation_when_dns_is_already_good() {
        let candidate = DnsCandidateResult {
            resolver: "1.1.1.1".to_string(),
            label: "Cloudflare".to_string(),
            latency_ms: 10.0,
            success: true,
        };
        let (should_apply, _) = evaluate_recommendation(Some(30.0), &Some(candidate));
        assert!(!should_apply);
    }

    #[test]
    fn resolve_interface_alias_falls_back_to_device_name() {
        let environment = EnvironmentSnapshot {
            hostname: "test".to_string(),
            interfaces: vec![NetworkInterface {
                name: "wlan0".to_string(),
                friendly_name: None,
                kind: LinkKind::WiFi,
                is_up: true,
                ipv4: vec!["192.168.1.10".to_string()],
                gateway: Some("192.168.1.1".to_string()),
                is_default_route: true,
            }],
            dns_servers: vec![],
            proxy: ProxySettings {
                enabled: false,
                server: None,
                source: "none".to_string(),
            },
            tags: vec![],
            default_gateway: Some("192.168.1.1".to_string()),
            active_interface: Some("wlan0".to_string()),
            tor: default_tor_status(),
        };

        let alias = resolve_interface_alias(&environment);
        assert!(alias.is_some());
        // Platform mapping may rewrite the label (e.g. macOS service name); never empty.
        assert!(!alias.unwrap().is_empty());
    }
}
