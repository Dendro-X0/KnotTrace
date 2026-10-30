//! Band C1 — hosts file + system proxy change snapshots for config timeline.

use crate::types::{
    AlertLevel, ConfigWatchEvent, HealthReport, HostsFileSnapshot, IntegrityBoardLevel,
    IntegrityBoardReport, IntegrityIndicator, IntegrityIndicatorKind, ProxySettings,
    ProxyWatchSnapshot,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_EVENT_LINES: usize = 200;

#[derive(Debug, Error)]
pub enum ConfigWatchError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigWatchState {
    hosts: HostsFileSnapshot,
    proxy: ProxyWatchSnapshot,
    updated_at: DateTime<Utc>,
}

pub fn state_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config_watch_state.json")
}

pub fn events_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config_watch_events.jsonl")
}

pub fn hosts_file_path() -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        return root.join(r"System32\drivers\etc\hosts");
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/etc/hosts")
    }
}

pub fn snapshot_hosts_file() -> HostsFileSnapshot {
    snapshot_hosts_at(&hosts_file_path())
}

pub fn snapshot_hosts_at(path: &Path) -> HostsFileSnapshot {
    let path_str = path.display().to_string();
    match std::fs::metadata(path).and_then(|meta| {
        let bytes = std::fs::read(path)?;
        let mtime_unix = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        Ok(HostsFileSnapshot {
            path: path_str.clone(),
            mtime_unix,
            size: meta.len(),
            content_hash: djb2_hex(&bytes),
            observed: true,
        })
    }) {
        Ok(snap) => snap,
        Err(_) => HostsFileSnapshot {
            path: path_str,
            mtime_unix: None,
            size: 0,
            content_hash: String::new(),
            observed: false,
        },
    }
}

pub fn snapshot_proxy(proxy: &ProxySettings) -> ProxyWatchSnapshot {
    let server = proxy.server.as_deref().unwrap_or("");
    let material = format!("{}|{}|{}", proxy.enabled, proxy.source, server);
    ProxyWatchSnapshot {
        enabled: proxy.enabled,
        server_fingerprint: djb2_hex(material.as_bytes()),
        source: proxy.source.clone(),
    }
}

fn djb2_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 5381;
    for &b in bytes {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(b));
    }
    format!("{hash:016x}")
}

fn load_state(data_dir: &Path) -> Result<Option<ConfigWatchState>, ConfigWatchError> {
    let path = state_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&raw)?))
}

fn save_state(data_dir: &Path, state: &ConfigWatchState) -> Result<(), ConfigWatchError> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(state_path(data_dir), serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn append_event(data_dir: &Path, event: &ConfigWatchEvent) -> Result<(), ConfigWatchError> {
    std::fs::create_dir_all(data_dir)?;
    let path = events_path(data_dir);
    let line = serde_json::to_string(event)?;

    let mut lines = if path.exists() {
        std::fs::read_to_string(&path)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    lines.push(line);
    if lines.len() > MAX_EVENT_LINES {
        lines.drain(0..lines.len() - MAX_EVENT_LINES);
    }

    let mut file = std::fs::File::create(path)?;
    for stored in lines {
        writeln!(file, "{stored}")?;
    }
    Ok(())
}

pub fn list_config_watch_events(
    data_dir: &Path,
    limit: usize,
) -> Result<Vec<ConfigWatchEvent>, ConfigWatchError> {
    let path = events_path(data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<ConfigWatchEvent>(&line) {
            entries.push(entry);
        }
    }

    let start = entries.len().saturating_sub(limit);
    Ok(entries[start..].to_vec())
}

fn hosts_changed(prev: &HostsFileSnapshot, current: &HostsFileSnapshot) -> bool {
    prev.observed
        && current.observed
        && (prev.content_hash != current.content_hash
            || prev.mtime_unix != current.mtime_unix
            || prev.size != current.size)
}

fn proxy_changed(prev: &ProxyWatchSnapshot, current: &ProxyWatchSnapshot) -> bool {
    prev.server_fingerprint != current.server_fingerprint
}

/// Compare current hosts/proxy to persisted state, append change events, update state,
/// and push integrity indicators when changes are first observed in this refresh.
pub fn refresh_config_watch(
    data_dir: &Path,
    report: &mut HealthReport,
) -> Result<Vec<ConfigWatchEvent>, ConfigWatchError> {
    let hosts = snapshot_hosts_file();
    let proxy = snapshot_proxy(&report.environment.proxy);
    let now = Utc::now();
    let mut new_events = Vec::new();

    if let Some(prior) = load_state(data_dir)? {
        if hosts_changed(&prior.hosts, &hosts) {
            let detail = format!(
                "Hosts file changed at {} (size {}→{}, hash {}→{})",
                hosts.path,
                prior.hosts.size,
                hosts.size,
                truncate_hash(&prior.hosts.content_hash),
                truncate_hash(&hosts.content_hash)
            );
            let event = ConfigWatchEvent {
                timestamp: now,
                kind: "hosts".to_string(),
                detail: detail.clone(),
            };
            append_event(data_dir, &event)?;
            push_hosts_indicator(report, &detail);
            new_events.push(event);
        }
        if proxy_changed(&prior.proxy, &proxy) {
            let detail = format!(
                "System proxy config changed (enabled {}→{}, source {}→{})",
                prior.proxy.enabled, proxy.enabled, prior.proxy.source, proxy.source
            );
            let event = ConfigWatchEvent {
                timestamp: now,
                kind: "proxy".to_string(),
                detail: detail.clone(),
            };
            append_event(data_dir, &event)?;
            push_proxy_delta_indicator(report, &detail);
            new_events.push(event);
        }
    }

    save_state(
        data_dir,
        &ConfigWatchState {
            hosts,
            proxy,
            updated_at: now,
        },
    )?;

    Ok(new_events)
}

fn truncate_hash(hash: &str) -> String {
    if hash.len() <= 10 {
        hash.to_string()
    } else {
        format!("{}…", &hash[..10])
    }
}

fn push_hosts_indicator(report: &mut HealthReport, detail: &str) {
    let Some(board) = report.integrity.as_mut() else {
        return;
    };
    if board
        .indicators
        .iter()
        .any(|i| i.kind == IntegrityIndicatorKind::HostsFileDelta)
    {
        return;
    }
    board.indicators.push(IntegrityIndicator {
        kind: IntegrityIndicatorKind::HostsFileDelta,
        severity: AlertLevel::Warning,
        title: "Hosts file changed".to_string(),
        message: format!(
            "{detail}. Local name overrides can redirect traffic — indicator only, not proof of malice."
        ),
        evidence: detail.to_string(),
    });
    bump_board_counts(board);
}

fn push_proxy_delta_indicator(report: &mut HealthReport, detail: &str) {
    let Some(board) = report.integrity.as_mut() else {
        return;
    };
    if board
        .indicators
        .iter()
        .any(|i| i.kind == IntegrityIndicatorKind::ProxyConfigDelta)
    {
        return;
    }
    board.indicators.push(IntegrityIndicator {
        kind: IntegrityIndicatorKind::ProxyConfigDelta,
        severity: AlertLevel::Warning,
        title: "System proxy configuration changed".to_string(),
        message: format!(
            "{detail}. Confirm the change matches a VPN/Clash client or intentional policy."
        ),
        evidence: detail.to_string(),
    });
    bump_board_counts(board);
}

fn bump_board_counts(board: &mut IntegrityBoardReport) {
    board.indicator_count = board.indicators.len().min(u8::MAX as usize) as u8;
    if matches!(
        board.level,
        IntegrityBoardLevel::None | IntegrityBoardLevel::Info
    ) {
        board.level = IntegrityBoardLevel::Caution;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn bare_report(proxy: ProxySettings) -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "test".to_string(),
                interfaces: vec![],
                dns_servers: vec![],
                proxy,
                tags: vec![],
                default_gateway: None,
                active_interface: None,
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: None,
                dns: Vec::new(),
                duration_ms: 0,
            },
            score: HealthScore {
                grade: HealthGrade::Good,
                score: 80,
                summary: "ok".to_string(),
                reasons: vec![],
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
            site_reachability: None,
            egress: None,
            network_context: None,
            recommendations: None,
            proxy_path_report: None,
            link_facts: None,
            local_caps: None,
            mtu_assist: None,
            local_repair: None,
            tunnel_compare: None,
            upstream_pool: None,
            path_break: None,
            integrity: Some(IntegrityBoardReport {
                indicator_count: 0,
                level: IntegrityBoardLevel::None,
                title: "none".to_string(),
                summary: "none".to_string(),
                disclaimer: crate::integrity_board::INTEGRITY_DISCLAIMER.to_string(),
                indicators: vec![],
                listeners: None,
                gateway_arp: None,
                cert_store: None,
                process_egress: None,
                not_checked: vec![],
                confidence: IntegrityBoardConfidence::Low,
            }),
            egress_timeline: None,
        }
    }

    #[test]
    fn first_refresh_seeds_without_events() {
        let dir = std::env::temp_dir().join(format!("knottrace-cw-seed-{}", uuid_like()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut report = bare_report(ProxySettings {
            enabled: false,
            server: None,
            source: "none".to_string(),
        });
        let events = refresh_config_watch(&dir, &mut report).unwrap();
        assert!(events.is_empty());
        assert!(state_path(&dir).exists());
        assert!(report
            .integrity
            .as_ref()
            .unwrap()
            .indicators
            .is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn proxy_flip_emits_event_and_indicator() {
        let dir = std::env::temp_dir().join(format!("knottrace-cw-proxy-{}", uuid_like()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut report = bare_report(ProxySettings {
            enabled: false,
            server: None,
            source: "none".to_string(),
        });
        refresh_config_watch(&dir, &mut report).unwrap();

        report.environment.proxy = ProxySettings {
            enabled: true,
            server: Some("127.0.0.1:7890".to_string()),
            source: "env:HTTPS_PROXY".to_string(),
        };
        let events = refresh_config_watch(&dir, &mut report).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "proxy");
        assert!(report
            .integrity
            .as_ref()
            .unwrap()
            .indicators
            .iter()
            .any(|i| i.kind == IntegrityIndicatorKind::ProxyConfigDelta));

        let listed = list_config_watch_events(&dir, 10).unwrap();
        assert_eq!(listed.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hosts_hash_change_emits_event() {
        let dir = std::env::temp_dir().join(format!("knottrace-cw-hosts-{}", uuid_like()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hosts = dir.join("hosts");
        std::fs::write(&hosts, b"127.0.0.1 localhost\n").unwrap();

        let prior = snapshot_hosts_at(&hosts);
        assert!(prior.observed);
        std::fs::write(&hosts, b"127.0.0.1 localhost\n10.0.0.1 evil\n").unwrap();
        let current = snapshot_hosts_at(&hosts);
        assert!(hosts_changed(&prior, &current));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn uuid_like() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}
