//! Opt-in process ↔ egress correlation (Path Truth Band 3E).

use crate::process::hidden_command;
use crate::types::*;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub const PROCESS_EGRESS_CONSENT: &str = "knottrace.process_egress.consent.v1";

#[derive(Debug, Error)]
pub enum ProcessEgressError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid settings: {0}")]
    Invalid(String),
}

pub fn settings_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("process_egress_settings.json")
}

pub fn default_process_egress_settings() -> ProcessEgressSettings {
    ProcessEgressSettings::default()
}

pub fn load_process_egress_settings(
    data_dir: &Path,
) -> Result<ProcessEgressSettings, ProcessEgressError> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(default_process_egress_settings());
    }
    let contents = std::fs::read_to_string(path)?;
    let settings: ProcessEgressSettings = serde_json::from_str(&contents)?;
    Ok(normalize_process_egress_settings(&settings))
}

pub fn save_process_egress_settings(
    data_dir: &Path,
    settings: &ProcessEgressSettings,
) -> Result<ProcessEgressSettings, ProcessEgressError> {
    let normalized = normalize_process_egress_settings(settings);
    if normalized.enabled && !normalized.consent_ack {
        return Err(ProcessEgressError::Invalid(
            "Ack consent before enabling process ↔ egress correlation.".to_string(),
        ));
    }
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        settings_path(data_dir),
        serde_json::to_string_pretty(&normalized)?,
    )?;
    Ok(normalized)
}

pub fn normalize_process_egress_settings(settings: &ProcessEgressSettings) -> ProcessEgressSettings {
    if settings.enabled && !settings.consent_ack {
        return ProcessEgressSettings {
            enabled: false,
            consent_ack: false,
        };
    }
    ProcessEgressSettings {
        enabled: settings.enabled,
        consent_ack: settings.consent_ack,
    }
}

/// Run opt-in process↔egress scan and merge into integrity board.
pub fn enrich_process_egress(report: &mut HealthReport, settings: &ProcessEgressSettings) {
    if !settings.enabled || !settings.consent_ack {
        return;
    }
    if report.integrity.is_none() {
        return;
    }

    let snapshot = scan_process_egress(report);
    apply_process_egress_snapshot(report, snapshot);
}

fn apply_process_egress_snapshot(report: &mut HealthReport, snapshot: ProcessEgressSnapshot) {
    let Some(board) = report.integrity.as_mut() else {
        return;
    };
    if snapshot.scanned && !snapshot.hits.is_empty() {
        let names: Vec<&str> = snapshot
            .hits
            .iter()
            .take(5)
            .map(|hit| hit.process_name.as_str())
            .collect();
        board.indicators.push(IntegrityIndicator {
            kind: IntegrityIndicatorKind::ProcessEgressHint,
            severity: AlertLevel::Info,
            title: "Processes with sessions toward current egress/proxy".to_string(),
            message: format!(
                "Matched TCP sessions for: {}. This is an indicator of which apps currently talk toward your observed egress/proxy — not proof of malware or interception.",
                names.join(", ")
            ),
            evidence: snapshot.summary.clone(),
        });
    }

    let listeners = board.listeners.clone();
    let gateway_arp = board.gateway_arp.clone();
    let cert_store = board.cert_store.clone();
    let indicators = board.indicators.clone();
    *board = crate::integrity_board::finalize_board_for_enrich(
        indicators,
        listeners,
        gateway_arp,
        cert_store,
        Some(snapshot),
    );
}

pub fn scan_process_egress(report: &HealthReport) -> ProcessEgressSnapshot {
    let targets = match_targets(report);
    if targets.is_empty() {
        return ProcessEgressSnapshot {
            scanned: true,
            enabled: true,
            source: "no_targets".to_string(),
            matched_targets: vec![],
            hits: vec![],
            summary: "No egress IP or proxy host available to match.".to_string(),
        };
    }

    #[cfg(windows)]
    {
        return scan_process_egress_windows(&targets);
    }
    #[cfg(target_os = "linux")]
    {
        return scan_process_egress_linux(&targets);
    }
    #[cfg(target_os = "macos")]
    {
        return scan_process_egress_macos(&targets);
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "not_implemented".to_string(),
            matched_targets: targets,
            hits: vec![],
            summary: "Process ↔ egress correlation is not implemented on this OS yet."
                .to_string(),
        }
    }
}

fn match_targets(report: &HealthReport) -> Vec<String> {
    let mut targets = Vec::new();
    if let Some(ip) = report
        .egress
        .as_ref()
        .and_then(|egress| egress.primary_ip.clone())
    {
        if !ip.is_empty() {
            targets.push(ip);
        }
    }
    if report.environment.proxy.enabled {
        if let Some(host) = proxy_host(report.environment.proxy.server.as_deref()) {
            if !targets.iter().any(|existing| existing == &host) {
                targets.push(host);
            }
        }
    }
    targets
}

fn proxy_host(server: Option<&str>) -> Option<String> {
    let raw = server?.trim();
    if raw.is_empty() {
        return None;
    }
    let without_scheme = raw
        .strip_prefix("http://")
        .or_else(|| raw.strip_prefix("https://"))
        .or_else(|| raw.strip_prefix("socks5://"))
        .or_else(|| raw.strip_prefix("socks4://"))
        .unwrap_or(raw);
    let host = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .split(':')
        .next()
        .unwrap_or(without_scheme)
        .trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

#[cfg(windows)]
fn scan_process_egress_windows(targets: &[String]) -> ProcessEgressSnapshot {
    let target_list = targets
        .iter()
        .map(|t| format!("'{t}'"))
        .collect::<Vec<_>>()
        .join(",");
    let ps = format!(
        "$targets = @({target_list}); Get-NetTCPConnection -State Established -ErrorAction SilentlyContinue | Where-Object {{ $targets -contains $_.RemoteAddress }} | Select-Object -First 80 OwningProcess,RemoteAddress | ForEach-Object {{ \"$($_.OwningProcess)`t$($_.RemoteAddress)\" }}"
    );
    let output = hidden_command("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let rows = parse_pid_remote_lines(&text);
            let hits = aggregate_hits(rows, resolve_process_name_windows);
            finish_snapshot(targets, hits, "powershell Get-NetTCPConnection")
        }
        Ok(out) => ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "powershell Get-NetTCPConnection".to_string(),
            matched_targets: targets.to_vec(),
            hits: vec![],
            summary: format!(
                "Process egress scan failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        },
        Err(error) => ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "powershell Get-NetTCPConnection".to_string(),
            matched_targets: targets.to_vec(),
            hits: vec![],
            summary: format!("Process egress scan unavailable: {error}"),
        },
    }
}

#[cfg(windows)]
fn resolve_process_name_windows(pid: u32) -> String {
    let ps = format!(
        "(Get-Process -Id {pid} -ErrorAction SilentlyContinue).ProcessName"
    );
    let output = hidden_command("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if name.is_empty() {
                format!("pid:{pid}")
            } else {
                name
            }
        }
        _ => format!("pid:{pid}"),
    }
}

#[cfg(target_os = "linux")]
fn scan_process_egress_linux(targets: &[String]) -> ProcessEgressSnapshot {
    let output = hidden_command("ss")
        .args(["-H", "-tnip"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let rows = parse_ss_tnip_lines(&text, targets);
            let hits = aggregate_hits(rows, |pid| resolve_process_name_proc(pid));
            finish_snapshot(targets, hits, "ss -tnip")
        }
        Ok(out) => ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "ss -tnip".to_string(),
            matched_targets: targets.to_vec(),
            hits: vec![],
            summary: format!(
                "Process egress scan failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        },
        Err(error) => ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "ss -tnip".to_string(),
            matched_targets: targets.to_vec(),
            hits: vec![],
            summary: format!("Process egress scan unavailable: {error}"),
        },
    }
}

#[cfg(target_os = "macos")]
fn scan_process_egress_macos(targets: &[String]) -> ProcessEgressSnapshot {
    let output = hidden_command("lsof")
        .args(["-nP", "-iTCP", "-sTCP:ESTABLISHED"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let rows = parse_lsof_tcp_lines(&text, targets);
            let hits = aggregate_hits(rows, |pid| resolve_process_name_proc(pid));
            finish_snapshot(targets, hits, "lsof -iTCP")
        }
        Ok(out) => ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "lsof -iTCP".to_string(),
            matched_targets: targets.to_vec(),
            hits: vec![],
            summary: format!(
                "Process egress scan failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        },
        Err(error) => ProcessEgressSnapshot {
            scanned: false,
            enabled: true,
            source: "lsof -iTCP".to_string(),
            matched_targets: targets.to_vec(),
            hits: vec![],
            summary: format!("Process egress scan unavailable: {error}"),
        },
    }
}

fn finish_snapshot(
    targets: &[String],
    hits: Vec<ProcessEgressHit>,
    source: &str,
) -> ProcessEgressSnapshot {
    let summary = if hits.is_empty() {
        format!(
            "No established TCP sessions matched targets {}.",
            targets.join(", ")
        )
    } else {
        format!(
            "{} process group(s) matched toward {}.",
            hits.len(),
            targets.join(", ")
        )
    };
    ProcessEgressSnapshot {
        scanned: true,
        enabled: true,
        source: source.to_string(),
        matched_targets: targets.to_vec(),
        hits,
        summary,
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn resolve_process_name_proc(pid: u32) -> String {
    let path = format!("/proc/{pid}/comm");
    if let Ok(name) = std::fs::read_to_string(&path) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    // macOS has no /proc; try ps
    let output = hidden_command("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    format!("pid:{pid}")
}

#[cfg_attr(windows, allow(dead_code))]
fn endpoint_host(endpoint: &str) -> String {
    if let Some(rest) = endpoint.strip_prefix('[') {
        if let Some((host, _)) = rest.split_once("]:") {
            return host.to_string();
        }
    }
    endpoint
        .rsplit_once(':')
        .map(|(host, _)| host.to_string())
        .unwrap_or_else(|| endpoint.to_string())
}

#[cfg_attr(windows, allow(dead_code))]
fn host_matches_targets(host: &str, targets: &[String]) -> bool {
    targets.iter().any(|target| target == host)
}

/// Parse `ss -H -tnip` lines; keep remotes matching targets.
#[cfg_attr(windows, allow(dead_code))]
fn parse_ss_tnip_lines(text: &str, targets: &[String]) -> Vec<(u32, String)> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let Some(remote) = ss_remote_address(line) else {
            continue;
        };
        let host = endpoint_host(&remote);
        if !host_matches_targets(&host, targets) {
            continue;
        }
        if let Some(pid) = ss_pid(line) {
            rows.push((pid, host));
        }
    }
    rows
}

#[cfg_attr(windows, allow(dead_code))]
fn ss_remote_address(line: &str) -> Option<String> {
    // Typical: ESTAB 0 0 192.168.1.2:443 203.0.113.10:443 users:(("curl",pid=123,fd=3))
    let before_users = line.split("users:").next().unwrap_or(line).trim();
    let parts: Vec<&str> = before_users.split_whitespace().collect();
    // Peer address is the last token before the users= blob.
    let candidate = parts.last().copied()?;
    if !candidate.contains(':') {
        return None;
    }
    Some(candidate.to_string())
}

#[cfg_attr(windows, allow(dead_code))]
fn ss_pid(line: &str) -> Option<u32> {
    let idx = line.find("pid=")?;
    let rest = &line[idx + 4..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Parse `lsof -nP -iTCP -sTCP:ESTABLISHED` lines for matching remotes.
#[cfg_attr(windows, allow(dead_code))]
fn parse_lsof_tcp_lines(text: &str, targets: &[String]) -> Vec<(u32, String)> {
    let mut rows = Vec::new();
    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }
        let Some(pid) = parts[1].parse::<u32>().ok() else {
            continue;
        };
        // e.g. 192.168.1.2:54321->203.0.113.10:443 (ESTABLISHED) — NAME may be split
        let endpoint = parts
            .iter()
            .find(|part| part.contains("->"))
            .copied()
            .unwrap_or("");
        let remote = endpoint.split("->").nth(1).unwrap_or("");
        if remote.is_empty() {
            continue;
        }
        let host = endpoint_host(remote);
        if !host_matches_targets(&host, targets) {
            continue;
        }
        rows.push((pid, host));
    }
    rows
}

fn parse_pid_remote_lines(text: &str) -> Vec<(u32, String)> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split('\t');
        let pid = parts.next().and_then(|p| p.trim().parse::<u32>().ok());
        let remote = parts.next().map(str::trim).filter(|s| !s.is_empty());
        if let (Some(pid), Some(remote)) = (pid, remote) {
            rows.push((pid, remote.to_string()));
        }
    }
    rows
}

fn aggregate_hits(
    rows: Vec<(u32, String)>,
    resolve_name: impl Fn(u32) -> String,
) -> Vec<ProcessEgressHit> {
    let mut grouped: HashMap<(u32, String), u8> = HashMap::new();
    for (pid, remote) in rows {
        let key = (pid, remote);
        let entry = grouped.entry(key).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    let mut hits: Vec<ProcessEgressHit> = grouped
        .into_iter()
        .map(|((pid, remote), count)| ProcessEgressHit {
            process_name: resolve_name(pid),
            pid: Some(pid),
            remote,
            count,
        })
        .collect();
    hits.sort_by(|a, b| b.count.cmp(&a.count).then(a.process_name.cmp(&b.process_name)));
    hits.truncate(12);
    hits
}

/// Test helper: apply a prebuilt snapshot (no OS scan).
#[cfg(test)]
fn enrich_process_egress_with_snapshot(
    report: &mut HealthReport,
    snapshot: ProcessEgressSnapshot,
) {
    apply_process_egress_snapshot(report, snapshot);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn base_report() -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "host".to_string(),
                interfaces: vec![],
                dns_servers: vec![],
                proxy: ProxySettings {
                    enabled: true,
                    server: Some("127.0.0.1:7897".to_string()),
                    source: "test".to_string(),
                },
                tags: vec![],
                default_gateway: Some("192.168.1.1".to_string()),
                active_interface: None,
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: None,
                dns: vec![],
                duration_ms: 0,
            },
            score: HealthScore {
                grade: HealthGrade::Good,
                score: 90,
                summary: "ok".to_string(),
                reasons: vec![],
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
            site_reachability: None,
            egress: Some(EgressReport {
                primary_ip: Some("203.0.113.10".to_string()),
                confidence: EgressConfidence::High,
                system_path: EgressPathReport {
                    kind: EgressPathKind::System,
                    primary_ip: Some("203.0.113.10".to_string()),
                    endpoints: vec![],
                    summary: "ok".to_string(),
                },
                tor_path: None,
                summary: "egress ok".to_string(),
            }),
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
                title: "No integrity indicators in this check".to_string(),
                summary: "none".to_string(),
                disclaimer: crate::integrity_board::INTEGRITY_DISCLAIMER.to_string(),
                indicators: vec![],
                listeners: None,
                gateway_arp: None,
                cert_store: None,
                process_egress: None,
                autoruns: None,
                not_checked: vec!["Process ↔ egress correlation".to_string()],
                confidence: IntegrityBoardConfidence::Low,
            }),
            egress_timeline: None,
        }
    }

    #[test]
    fn defaults_disabled() {
        let settings = default_process_egress_settings();
        assert!(!settings.enabled);
        assert!(!settings.consent_ack);
    }

    #[test]
    fn proxy_host_parses() {
        assert_eq!(
            proxy_host(Some("http://127.0.0.1:7897")).as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(
            proxy_host(Some("socks5://10.0.0.2:1080")).as_deref(),
            Some("10.0.0.2")
        );
    }

    #[test]
    fn match_targets_include_egress_and_proxy() {
        let targets = match_targets(&base_report());
        assert!(targets.iter().any(|t| t == "203.0.113.10"));
        assert!(targets.iter().any(|t| t == "127.0.0.1"));
    }

    #[test]
    fn parse_and_aggregate_hits() {
        let rows = parse_pid_remote_lines("100\t203.0.113.10\n100\t203.0.113.10\n200\t127.0.0.1\n");
        assert_eq!(rows.len(), 3);
        let hits = aggregate_hits(rows, |pid| format!("proc-{pid}"));
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].count, 2);
    }

    #[test]
    fn enrich_with_snapshot_adds_indicator_and_clears_not_checked() {
        let mut report = base_report();
        enrich_process_egress_with_snapshot(
            &mut report,
            ProcessEgressSnapshot {
                scanned: true,
                enabled: true,
                source: "test".to_string(),
                matched_targets: vec!["203.0.113.10".to_string()],
                hits: vec![ProcessEgressHit {
                    process_name: "chrome".to_string(),
                    pid: Some(1),
                    remote: "203.0.113.10".to_string(),
                    count: 3,
                }],
                summary: "1 process group matched".to_string(),
            },
        );
        let board = report.integrity.unwrap();
        assert!(board
            .indicators
            .iter()
            .any(|i| i.kind == IntegrityIndicatorKind::ProcessEgressHint));
        assert!(board.process_egress.as_ref().is_some_and(|s| s.scanned));
        assert!(!board
            .not_checked
            .iter()
            .any(|s| s.contains("Process ↔ egress")));
    }

    #[test]
    fn disabled_enrich_is_noop() {
        let mut report = base_report();
        enrich_process_egress(
            &mut report,
            &ProcessEgressSettings {
                enabled: false,
                consent_ack: false,
            },
        );
        assert!(report
            .integrity
            .as_ref()
            .unwrap()
            .not_checked
            .iter()
            .any(|s| s.contains("Process ↔ egress")));
    }

    #[test]
    fn parses_ss_and_lsof_for_matching_targets() {
        let targets = vec!["203.0.113.10".to_string()];
        let ss = "ESTAB 0 0 192.168.1.2:443 203.0.113.10:443 users:((curl,pid=4242,fd=3))\n";
        assert_eq!(
            ss_remote_address(ss.trim()).as_deref(),
            Some("203.0.113.10:443")
        );
        assert_eq!(ss_pid(ss.trim()), Some(4242));
        let ss_rows = parse_ss_tnip_lines(ss, &targets);
        assert_eq!(ss_rows, vec![(4242, "203.0.113.10".to_string())]);

        let lsof = "COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\n\
curl      4242 me     3u   IPv4 0x0      0t0  TCP 192.168.1.2:54321->203.0.113.10:443 (ESTABLISHED)\n";
        let lsof_rows = parse_lsof_tcp_lines(lsof, &targets);
        assert_eq!(lsof_rows, vec![(4242, "203.0.113.10".to_string())]);
    }
}
