//! Integrity / exposure board (Path Truth Band 3) — indicators only.

use crate::process::hidden_command;
use crate::types::*;
use std::collections::BTreeSet;
use std::net::Ipv4Addr;

pub const INTEGRITY_DISCLAIMER: &str = "We found indicators consistent with exposure, forced routing, or interception. Absence of indicators does not mean you are unobserved.";

const HIGH_RISK_PORTS: &[u16] = &[21, 23, 135, 139, 445, 3389, 5900, 5985, 5986, 5600];
const NOT_CHECKED_PROCESS: &str = "Process ↔ egress correlation";
const NOT_CHECKED_COVERT: &str = "Covert kernel / firmware / ISP taps";
const NOT_CHECKED_ARP: &str = "Gateway / ARP identity churn over time";
const NOT_CHECKED_CERT: &str = "Trusted root certificate store deltas";

pub fn assess_integrity_board(report: &HealthReport) -> IntegrityBoardReport {
    assess_integrity_board_with_parts(
        report,
        scan_listeners(),
        snapshot_gateway_arp(report.environment.default_gateway.as_deref()),
        snapshot_cert_store(),
    )
}

/// Test/helper entry that skips live ARP/cert OS probes.
#[allow(dead_code)]
pub fn assess_integrity_board_with_listeners(
    report: &HealthReport,
    listeners: Option<ListenerScanSummary>,
) -> IntegrityBoardReport {
    assess_integrity_board_with_parts(report, listeners, None, None)
}

pub fn assess_integrity_board_with_parts(
    report: &HealthReport,
    listeners: Option<ListenerScanSummary>,
    gateway_arp: Option<GatewayArpSnapshot>,
    cert_store: Option<CertStoreSnapshot>,
) -> IntegrityBoardReport {
    let mut indicators = Vec::new();
    push_proxy_indicator(report, &mut indicators);
    push_dns_indicator(report, &mut indicators);
    push_captive_indicator(report, &mut indicators);
    push_tls_hint(report, &mut indicators);
    push_forced_routing(report, &mut indicators);

    if let Some(scan) = &listeners {
        for entry in &scan.high_risk {
            indicators.push(IntegrityIndicator {
                kind: IntegrityIndicatorKind::HighRiskListener,
                severity: AlertLevel::Warning,
                title: format!("High-risk listen on port {}", entry.port),
                message: format!(
                    "Local service listening on {} (port {}). This is an exposure indicator, not proof of compromise.",
                    entry.address, entry.port
                ),
                evidence: entry
                    .process_hint
                    .clone()
                    .unwrap_or_else(|| format!("{}:{}", entry.address, entry.port)),
            });
        }
    }

    finalize_board(indicators, listeners, gateway_arp, cert_store)
}

/// Apply Band 3D history deltas (ARP MAC churn, cert-store fingerprint change).
pub fn enrich_integrity_board(report: &mut HealthReport, history: &[HealthReport]) {
    let Some(board) = report.integrity.as_mut() else {
        return;
    };

    if let (Some(current), Some(mac)) = (
        board.gateway_arp.clone(),
        board.gateway_arp.as_ref().and_then(|s| s.mac.clone()),
    ) {
        if current.observed {
            for prior in history.iter().filter_map(|r| r.integrity.as_ref()) {
                let Some(prev) = &prior.gateway_arp else {
                    continue;
                };
                if !prev.observed || prev.gateway_ip != current.gateway_ip {
                    continue;
                }
                let Some(prev_mac) = &prev.mac else {
                    continue;
                };
                if prev_mac != &mac {
                    board.indicators.push(IntegrityIndicator {
                        kind: IntegrityIndicatorKind::GatewayArpChurn,
                        severity: AlertLevel::Warning,
                        title: "Gateway ARP identity changed".to_string(),
                        message: format!(
                            "Default gateway {} previously resolved to MAC {}, now {}. Router swap, network move, or ARP spoofing are all possible — treat as an indicator only.",
                            current.gateway_ip, prev_mac, mac
                        ),
                        evidence: format!(
                            "gateway={} previous_mac={} current_mac={}",
                            current.gateway_ip, prev_mac, mac
                        ),
                    });
                    break;
                }
            }
        }
    }

    if let Some(current) = board.cert_store.clone() {
        if current.observed {
            let prior_cert = history.iter().find_map(|r| {
                r.integrity.as_ref().and_then(|b| {
                    b.cert_store
                        .as_ref()
                        .filter(|c| c.observed)
                        .cloned()
                })
            });
            if let Some(prev) = prior_cert {
                if prev.fingerprint != current.fingerprint || prev.root_count != current.root_count
                {
                    board.indicators.push(IntegrityIndicator {
                        kind: IntegrityIndicatorKind::CertStoreDelta,
                        severity: AlertLevel::Warning,
                        title: "Trusted root store changed".to_string(),
                        message: format!(
                            "LocalMachine trusted roots were {} (fingerprint {}), now {} ({}). New enterprise/MITM roots or OS updates can both cause this — indicator only.",
                            prev.root_count,
                            truncate_fp(&prev.fingerprint),
                            current.root_count,
                            truncate_fp(&current.fingerprint)
                        ),
                        evidence: format!(
                            "previous_count={} previous_fp={} current_count={} current_fp={}",
                            prev.root_count,
                            prev.fingerprint,
                            current.root_count,
                            current.fingerprint
                        ),
                    });
                }
            }
        }
    }

    let listeners = board.listeners.clone();
    let gateway_arp = board.gateway_arp.clone();
    let cert_store = board.cert_store.clone();
    let indicators = board.indicators.clone();
    *board = finalize_board(indicators, listeners, gateway_arp, cert_store);
}

fn truncate_fp(fp: &str) -> String {
    if fp.len() <= 12 {
        fp.to_string()
    } else {
        format!("{}…", &fp[..12])
    }
}

fn finalize_board(
    indicators: Vec<IntegrityIndicator>,
    listeners: Option<ListenerScanSummary>,
    gateway_arp: Option<GatewayArpSnapshot>,
    cert_store: Option<CertStoreSnapshot>,
) -> IntegrityBoardReport {
    let indicator_count = indicators.len().min(u8::MAX as usize) as u8;
    let level = classify_level(&indicators);
    let confidence = if listeners.as_ref().map(|l| l.scanned).unwrap_or(false)
        && !indicators.is_empty()
    {
        IntegrityBoardConfidence::High
    } else if !indicators.is_empty() {
        IntegrityBoardConfidence::Medium
    } else {
        IntegrityBoardConfidence::Low
    };

    let (title, summary) = if indicator_count == 0 {
        (
            "No integrity indicators in this check".to_string(),
            "No proxy, DNS integrity, captive, TLS-intercept-hint, high-risk listener, ARP-churn, or cert-delta indicators fired. This is not a clean bill of health.".to_string(),
        )
    } else {
        (
            format!("{indicator_count} integrity indicator(s)"),
            format!(
                "{} See evidence below. {}",
                indicators
                    .first()
                    .map(|i| i.title.as_str())
                    .unwrap_or("Indicators present."),
                INTEGRITY_DISCLAIMER
            ),
        )
    };

    let mut not_checked = vec![
        NOT_CHECKED_PROCESS.to_string(),
        NOT_CHECKED_COVERT.to_string(),
    ];
    if !gateway_arp.as_ref().is_some_and(|s| s.observed) {
        not_checked.insert(0, NOT_CHECKED_ARP.to_string());
    }
    if !cert_store.as_ref().is_some_and(|s| s.observed) {
        not_checked.insert(0, NOT_CHECKED_CERT.to_string());
    }

    IntegrityBoardReport {
        indicator_count,
        level,
        title,
        summary,
        disclaimer: INTEGRITY_DISCLAIMER.to_string(),
        indicators,
        listeners,
        gateway_arp,
        cert_store,
        not_checked,
        confidence,
    }
}

fn snapshot_gateway_arp(gateway_ip: Option<&str>) -> Option<GatewayArpSnapshot> {
    let Some(ip) = gateway_ip.filter(|s| !s.is_empty()) else {
        return Some(GatewayArpSnapshot {
            gateway_ip: String::new(),
            mac: None,
            source: "no_default_gateway".to_string(),
            observed: false,
        });
    };

    #[cfg(windows)]
    {
        return Some(snapshot_gateway_arp_windows(ip));
    }
    #[cfg(target_os = "linux")]
    {
        return Some(snapshot_gateway_arp_linux(ip));
    }
    #[cfg(target_os = "macos")]
    {
        return Some(snapshot_gateway_arp_macos(ip));
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = ip;
        Some(GatewayArpSnapshot {
            gateway_ip: ip.to_string(),
            mac: None,
            source: "unsupported_os".to_string(),
            observed: false,
        })
    }
}

fn snapshot_cert_store() -> Option<CertStoreSnapshot> {
    #[cfg(windows)]
    {
        return Some(snapshot_cert_store_windows());
    }
    #[cfg(not(windows))]
    {
        Some(CertStoreSnapshot {
            root_count: 0,
            fingerprint: String::new(),
            source: "cert_store_not_implemented".to_string(),
            observed: false,
        })
    }
}

fn normalize_mac(raw: &str) -> Option<String> {
    let hex: String = raw
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if hex.len() != 12 {
        return None;
    }
    Some(
        hex.as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).unwrap_or("00"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}

fn fingerprint_thumbprints(thumbs: &[String]) -> String {
    let mut sorted = thumbs.to_vec();
    sorted.sort();
    let joined = sorted.join("|");
    let mut hash: u64 = 5381;
    for b in joined.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(b));
    }
    format!("{:016x}:{}", hash, sorted.len())
}

#[cfg(windows)]
fn snapshot_gateway_arp_windows(gateway_ip: &str) -> GatewayArpSnapshot {
    let ps = format!(
        "$n = Get-NetNeighbor -IPAddress '{gateway_ip}' -ErrorAction SilentlyContinue | Select-Object -First 1; if ($n) {{ $n.LinkLayerAddress }} else {{ '' }}"
    );
    let output = hidden_command("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(mac) = text.lines().find_map(|line| normalize_mac(line.trim())) {
                return GatewayArpSnapshot {
                    gateway_ip: gateway_ip.to_string(),
                    mac: Some(mac),
                    source: "powershell Get-NetNeighbor".to_string(),
                    observed: true,
                };
            }
        }
    }

    // Fallback: arp -a
    let arp = hidden_command("arp").args(["-a"]).output();
    if let Ok(out) = arp {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(mac) = parse_arp_a_for_ip(&text, gateway_ip) {
                return GatewayArpSnapshot {
                    gateway_ip: gateway_ip.to_string(),
                    mac: Some(mac),
                    source: "arp -a".to_string(),
                    observed: true,
                };
            }
        }
    }

    GatewayArpSnapshot {
        gateway_ip: gateway_ip.to_string(),
        mac: None,
        source: "Get-NetNeighbor+arp".to_string(),
        observed: false,
    }
}

#[cfg(target_os = "linux")]
fn snapshot_gateway_arp_linux(gateway_ip: &str) -> GatewayArpSnapshot {
    match std::fs::read_to_string("/proc/net/arp") {
        Ok(text) => {
            if let Some(mac) = parse_proc_net_arp(&text, gateway_ip) {
                GatewayArpSnapshot {
                    gateway_ip: gateway_ip.to_string(),
                    mac: Some(mac),
                    source: "/proc/net/arp".to_string(),
                    observed: true,
                }
            } else {
                GatewayArpSnapshot {
                    gateway_ip: gateway_ip.to_string(),
                    mac: None,
                    source: "/proc/net/arp".to_string(),
                    observed: false,
                }
            }
        }
        Err(_) => GatewayArpSnapshot {
            gateway_ip: gateway_ip.to_string(),
            mac: None,
            source: "/proc/net/arp".to_string(),
            observed: false,
        },
    }
}

#[cfg(target_os = "macos")]
fn snapshot_gateway_arp_macos(gateway_ip: &str) -> GatewayArpSnapshot {
    let output = hidden_command("arp").args(["-n", gateway_ip]).output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(mac) = text.split_whitespace().find_map(|tok| normalize_mac(tok)) {
                GatewayArpSnapshot {
                    gateway_ip: gateway_ip.to_string(),
                    mac: Some(mac),
                    source: "arp -n".to_string(),
                    observed: true,
                }
            } else {
                GatewayArpSnapshot {
                    gateway_ip: gateway_ip.to_string(),
                    mac: None,
                    source: "arp -n".to_string(),
                    observed: false,
                }
            }
        }
        _ => GatewayArpSnapshot {
            gateway_ip: gateway_ip.to_string(),
            mac: None,
            source: "arp -n".to_string(),
            observed: false,
        },
    }
}

#[cfg(windows)]
fn snapshot_cert_store_windows() -> CertStoreSnapshot {
    let ps = "(Get-ChildItem Cert:\\LocalMachine\\Root -ErrorAction SilentlyContinue | ForEach-Object { $_.Thumbprint }) -join \"`n\"";
    let output = hidden_command("powershell")
        .args(["-NoProfile", "-Command", ps])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let thumbs: Vec<String> = text
                .lines()
                .map(|l| l.trim().to_ascii_uppercase())
                .filter(|l| !l.is_empty() && l.chars().all(|c| c.is_ascii_hexdigit()))
                .collect();
            if thumbs.is_empty() {
                return CertStoreSnapshot {
                    root_count: 0,
                    fingerprint: String::new(),
                    source: "Cert:\\LocalMachine\\Root".to_string(),
                    observed: false,
                };
            }
            let root_count = thumbs.len().min(u16::MAX as usize) as u16;
            let fingerprint = fingerprint_thumbprints(&thumbs);
            CertStoreSnapshot {
                root_count,
                fingerprint,
                source: "Cert:\\LocalMachine\\Root".to_string(),
                observed: true,
            }
        }
        _ => CertStoreSnapshot {
            root_count: 0,
            fingerprint: String::new(),
            source: "Cert:\\LocalMachine\\Root".to_string(),
            observed: false,
        },
    }
}

#[cfg_attr(not(windows), allow(dead_code))]
fn parse_arp_a_for_ip(text: &str, gateway_ip: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(gateway_ip) {
            continue;
        }
        for tok in trimmed.split_whitespace() {
            if let Some(mac) = normalize_mac(tok) {
                return Some(mac);
            }
        }
    }
    None
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_proc_net_arp(text: &str, gateway_ip: &str) -> Option<String> {
    for line in text.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let ip = parts.next()?;
        if ip != gateway_ip {
            continue;
        }
        let _hw_type = parts.next()?;
        let _flags = parts.next()?;
        let mac = parts.next()?;
        return normalize_mac(mac);
    }
    None
}

fn push_proxy_indicator(report: &HealthReport, indicators: &mut Vec<IntegrityIndicator>) {
    if !report.environment.proxy.enabled {
        return;
    }
    let server = report
        .environment
        .proxy
        .server
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    indicators.push(IntegrityIndicator {
        kind: IntegrityIndicatorKind::ProxyEnabled,
        severity: AlertLevel::Info,
        title: "System proxy enabled".to_string(),
        message: "HTTP(S)/SOCKS proxy is enabled. Confirm this matches your intent (VPN client, Clash, corporate PAC).".to_string(),
        evidence: format!("proxy server={server} source={}", report.environment.proxy.source),
    });
}

fn push_dns_indicator(report: &HealthReport, indicators: &mut Vec<IntegrityIndicator>) {
    let Some(dns) = &report.dns_integrity else {
        return;
    };
    let (severity, label) = match dns.state {
        DnsIntegrityState::Ok => return,
        DnsIntegrityState::Caution => (AlertLevel::Warning, "DNS integrity caution"),
        DnsIntegrityState::Suspicious => (AlertLevel::Critical, "DNS integrity suspicious"),
    };
    indicators.push(IntegrityIndicator {
        kind: IntegrityIndicatorKind::DnsIntegrity,
        severity,
        title: label.to_string(),
        message: dns.summary.clone(),
        evidence: format!(
            "mismatches={} checked={}",
            dns.mismatch_count, dns.checked_domains
        ),
    });
}

fn push_captive_indicator(report: &HealthReport, indicators: &mut Vec<IntegrityIndicator>) {
    let Some(ctx) = &report.network_context else {
        return;
    };
    match ctx.captive_portal.state {
        CaptivePortalState::NotDetected => {}
        CaptivePortalState::Suspected => indicators.push(IntegrityIndicator {
            kind: IntegrityIndicatorKind::CaptivePortal,
            severity: AlertLevel::Warning,
            title: "Captive portal suspected".to_string(),
            message: ctx.captive_portal.summary.clone(),
            evidence: ctx.captive_portal.probe_url.clone(),
        }),
        CaptivePortalState::Confirmed => indicators.push(IntegrityIndicator {
            kind: IntegrityIndicatorKind::CaptivePortal,
            severity: AlertLevel::Critical,
            title: "Captive portal confirmed".to_string(),
            message: ctx.captive_portal.summary.clone(),
            evidence: ctx.captive_portal.probe_url.clone(),
        }),
    }
}

fn push_tls_hint(report: &HealthReport, indicators: &mut Vec<IntegrityIndicator>) {
    let tls_failures = count_tls_failures(report);
    if tls_failures < 2 {
        return;
    }
    let overlay = report.environment.proxy.enabled
        || report.environment.tags.contains(&EnvironmentTag::Vpn)
        || report.environment.tor.detected;
    if !overlay {
        return;
    }
    indicators.push(IntegrityIndicator {
        kind: IntegrityIndicatorKind::TlsInterceptHint,
        severity: AlertLevel::Warning,
        title: "TLS failures on tunneled path".to_string(),
        message: "Multiple TLS errors while proxy/VPN/Tor is present can indicate interception, filtering, or a broken tunnel — treat as an indicator only.".to_string(),
        evidence: format!("tls_failure_samples={tls_failures}"),
    });
}

fn count_tls_failures(report: &HealthReport) -> usize {
    let mut n = 0;
    if let Some(reach) = &report.site_reachability {
        n += reach
            .results
            .iter()
            .filter(|r| !r.success && r.error_kind == Some(SiteReachErrorKind::Tls))
            .count();
    }
    if let Some(compare) = &report.tunnel_compare {
        for path in &compare.paths {
            n += path
                .reachability
                .iter()
                .filter(|r| !r.success && r.error_kind == Some(SiteReachErrorKind::Tls))
                .count();
        }
    }
    n
}

fn push_forced_routing(report: &HealthReport, indicators: &mut Vec<IntegrityIndicator>) {
    let Some(pb) = &report.path_break else {
        return;
    };
    if matches!(
        pb.active_path,
        PathBreakPath::SystemProxy | PathBreakPath::VpnTunnel
    ) && report.environment.proxy.enabled
    {
        // Avoid duplicating if proxy indicator already covers the same story with no break.
        if pb.break_hop != PathBreakHop::None && pb.break_hop != PathBreakHop::Local {
            indicators.push(IntegrityIndicator {
                kind: IntegrityIndicatorKind::PathForcedRouting,
                severity: AlertLevel::Warning,
                title: "Active path steered through proxy/VPN".to_string(),
                message: format!(
                    "Path break reports {} failing at {:?}. Traffic may be forced through an overlay.",
                    match pb.active_path {
                        PathBreakPath::SystemProxy => "system proxy",
                        PathBreakPath::VpnTunnel => "VPN",
                        _ => "overlay",
                    },
                    pb.break_hop
                ),
                evidence: pb.title.clone(),
            });
        }
    }
}

fn classify_level(indicators: &[IntegrityIndicator]) -> IntegrityBoardLevel {
    if indicators.is_empty() {
        return IntegrityBoardLevel::None;
    }
    if indicators.iter().any(|i| i.severity == AlertLevel::Critical) {
        return IntegrityBoardLevel::Elevated;
    }
    if indicators.iter().any(|i| i.severity == AlertLevel::Warning) {
        return IntegrityBoardLevel::Caution;
    }
    IntegrityBoardLevel::Info
}

fn scan_listeners() -> Option<ListenerScanSummary> {
    #[cfg(windows)]
    {
        return Some(scan_listeners_windows());
    }
    #[cfg(target_os = "linux")]
    {
        return Some(scan_listeners_linux());
    }
    #[cfg(target_os = "macos")]
    {
        return Some(scan_listeners_macos());
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(windows)]
fn scan_listeners_windows() -> ListenerScanSummary {
    let output = hidden_command("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Select-Object -First 80 LocalAddress,LocalPort | ForEach-Object { \"$($_.LocalAddress)`t$($_.LocalPort)\" }",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            summarize_listener_rows(parse_address_port_lines(&text), "powershell Get-NetTCPConnection")
        }
        Ok(out) => ListenerScanSummary {
            scanned: false,
            source: "powershell Get-NetTCPConnection".to_string(),
            high_risk: vec![],
            total_listened_sampled: 0,
            summary: format!(
                "Listener scan failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        },
        Err(error) => ListenerScanSummary {
            scanned: false,
            source: "powershell Get-NetTCPConnection".to_string(),
            high_risk: vec![],
            total_listened_sampled: 0,
            summary: format!("Listener scan unavailable: {error}"),
        },
    }
}

#[cfg(target_os = "linux")]
fn scan_listeners_linux() -> ListenerScanSummary {
    let mut rows = Vec::new();
    for path in ["/proc/net/tcp", "/proc/net/tcp6"] {
        if let Ok(text) = std::fs::read_to_string(path) {
            rows.extend(parse_proc_net_tcp(&text));
        }
    }
    if rows.is_empty() {
        return ListenerScanSummary {
            scanned: false,
            source: "/proc/net/tcp".to_string(),
            high_risk: vec![],
            total_listened_sampled: 0,
            summary: "Could not read /proc/net/tcp listener table.".to_string(),
        };
    }
    summarize_listener_rows(rows, "/proc/net/tcp")
}

#[cfg(target_os = "macos")]
fn scan_listeners_macos() -> ListenerScanSummary {
    let output = hidden_command("netstat")
        .args(["-an", "-p", "tcp"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            summarize_listener_rows(parse_netstat_listen_lines(&text), "netstat -an -p tcp")
        }
        Ok(_) | Err(_) => ListenerScanSummary {
            scanned: false,
            source: "netstat".to_string(),
            high_risk: vec![],
            total_listened_sampled: 0,
            summary: "Listener scan via netstat failed.".to_string(),
        },
    }
}

fn summarize_listener_rows(
    rows: Vec<(String, u16)>,
    source: &str,
) -> ListenerScanSummary {
    let sampled = rows.len().min(u8::MAX as usize) as u8;
    let mut high_risk = Vec::new();
    let mut seen = BTreeSet::new();
    for (address, port) in rows {
        if !is_high_risk_port(port) || is_loopback_address(&address) {
            continue;
        }
        let key = format!("{address}:{port}");
        if !seen.insert(key) {
            continue;
        }
        high_risk.push(HighRiskListener {
            port,
            address,
            process_hint: None,
        });
        if high_risk.len() >= 12 {
            break;
        }
    }
    let summary = if high_risk.is_empty() {
        format!("Scanned listen sockets via {source}; no high-risk non-loopback ports in sample.")
    } else {
        format!(
            "Found {} high-risk non-loopback listen port(s) via {source}.",
            high_risk.len()
        )
    };
    ListenerScanSummary {
        scanned: true,
        source: source.to_string(),
        high_risk,
        total_listened_sampled: sampled,
        summary,
    }
}

fn is_high_risk_port(port: u16) -> bool {
    HIGH_RISK_PORTS.contains(&port)
}

fn is_loopback_address(address: &str) -> bool {
    let trimmed = address.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed == "::1" || trimmed.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Ok(ip) = trimmed.parse::<Ipv4Addr>() {
        return ip.is_loopback();
    }
    trimmed.starts_with("127.")
}

/// Parse lines shaped like `address\\tport` or `address port`.
pub fn parse_address_port_lines(text: &str) -> Vec<(String, u16)> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (addr, port_str) = if let Some((a, p)) = line.split_once('\t') {
            (a.trim(), p.trim())
        } else if let Some((a, p)) = line.rsplit_once(' ') {
            (a.trim(), p.trim())
        } else if let Some((a, p)) = line.rsplit_once(':') {
            // Avoid splitting IPv6 carelessly — only for simple IPv4 host:port
            if a.matches(':').count() == 0 {
                (a.trim(), p.trim())
            } else {
                continue;
            }
        } else {
            continue;
        };
        if let Ok(port) = port_str.parse::<u16>() {
            rows.push((addr.to_string(), port));
        }
    }
    rows
}

/// Parse Linux `/proc/net/tcp` listen entries (state 0A).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn parse_proc_net_tcp(text: &str) -> Vec<(String, u16)> {
    let mut rows = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        // state listen = 0A
        if cols[3] != "0A" {
            continue;
        }
        if let Some((addr, port)) = parse_hex_endpoint(cols[1]) {
            rows.push((addr, port));
        }
    }
    rows
}

fn parse_hex_endpoint(raw: &str) -> Option<(String, u16)> {
    let (ip_hex, port_hex) = raw.split_once(':')?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    if ip_hex.len() == 8 {
        let ip_num = u32::from_str_radix(ip_hex, 16).ok()?;
        let b = ip_num.to_le_bytes();
        Some((format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]), port))
    } else {
        // IPv6 compressed display — keep hex form for address identity
        Some((format!("[{ip_hex}]"), port))
    }
}

/// Parse macOS `netstat -an -p tcp` listen lines.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn parse_netstat_listen_lines(text: &str) -> Vec<(String, u16)> {
    let mut rows = Vec::new();
    for line in text.lines() {
        if !line.contains("LISTEN") {
            continue;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        let local = cols[3];
        if let Some((addr, port)) = split_host_port(local) {
            rows.push((addr, port));
        }
    }
    rows
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn split_host_port(local: &str) -> Option<(String, u16)> {
    if let Some(stripped) = local.strip_prefix("*.") {
        let port = stripped.parse().ok()?;
        return Some(("0.0.0.0".to_string(), port));
    }
    if let Some((host, port)) = local.rsplit_once('.') {
        // netstat often uses 127.0.0.1.445
        if let Ok(p) = port.parse::<u16>() {
            return Some((host.to_string(), p));
        }
    }
    if let Some((host, port)) = local.rsplit_once(':') {
        if host.matches(':').count() <= 1 {
            if let Ok(p) = port.parse::<u16>() {
                return Some((host.trim_matches(['[', ']']).to_string(), p));
            }
        }
    }
    None
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
                    enabled: false,
                    server: None,
                    source: "none".to_string(),
                },
                tags: vec![],
                default_gateway: None,
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
            integrity: None,
            egress_timeline: None,
        }
    }

    fn empty_listeners() -> ListenerScanSummary {
        ListenerScanSummary {
            scanned: true,
            source: "test".to_string(),
            high_risk: vec![],
            total_listened_sampled: 0,
            summary: "none".to_string(),
        }
    }

    #[test]
    fn zero_indicators_are_not_safe_copy() {
        let board =
            assess_integrity_board_with_listeners(&base_report(), Some(empty_listeners()));
        assert_eq!(board.indicator_count, board.indicators.len() as u8);
        assert_eq!(board.level, IntegrityBoardLevel::None);
        assert!(board.title.contains("No integrity indicators"));
        assert!(board.summary.contains("not a clean bill"));
        assert_eq!(board.disclaimer, INTEGRITY_DISCLAIMER);
        assert!(!board.not_checked.is_empty());
    }

    #[test]
    fn proxy_and_dns_become_indicators() {
        let mut report = base_report();
        report.environment.proxy.enabled = true;
        report.environment.proxy.server = Some("127.0.0.1:7890".to_string());
        report.dns_integrity = Some(DnsIntegrityStatus {
            state: DnsIntegrityState::Suspicious,
            confidence: DnsIntegrityConfidence::High,
            mismatch_count: 2,
            checked_domains: 4,
            summary: "Answers diverge.".to_string(),
            details: vec![],
        });
        let board = assess_integrity_board_with_listeners(&report, Some(empty_listeners()));
        assert!(board.indicator_count >= 2);
        assert_eq!(board.level, IntegrityBoardLevel::Elevated);
        assert!(board
            .indicators
            .iter()
            .any(|i| i.kind == IntegrityIndicatorKind::ProxyEnabled));
        assert!(board
            .indicators
            .iter()
            .any(|i| i.kind == IntegrityIndicatorKind::DnsIntegrity));
    }

    #[test]
    fn parses_proc_net_tcp_listen() {
        // 0100007F:0050 = 127.0.0.1:80 ; 00000000:01BB = 0.0.0.0:443 — use high-risk 0151=337 / wait use 0D3D=3389
        // 3389 = 0D3D
        let text = "  sl  local_address rem_address   st\n \
   0: 00000000:0D3D 00000000:0000 0A 00000000:00000000 00:00000000 00000000\n \
   1: 0100007F:0050 00000000:0000 0A 00000000:00000000 00:00000000 00000000\n";
        let rows = parse_proc_net_tcp(text);
        assert!(rows.iter().any(|(addr, port)| addr == "0.0.0.0" && *port == 3389));
        let summary = summarize_listener_rows(rows, "test");
        assert!(summary.scanned);
        assert_eq!(summary.high_risk.len(), 1);
        assert_eq!(summary.high_risk[0].port, 3389);
    }

    #[test]
    fn parse_address_port_lines_basic() {
        let rows = parse_address_port_lines("0.0.0.0\t445\n127.0.0.1\t3389\n");
        assert_eq!(rows.len(), 2);
        let summary = summarize_listener_rows(rows, "test");
        // 127 loopback filtered; 445 on 0.0.0.0 kept
        assert_eq!(summary.high_risk.len(), 1);
        assert_eq!(summary.high_risk[0].port, 445);
    }

    #[test]
    fn normalizes_mac_and_parses_arp_tables() {
        assert_eq!(
            normalize_mac("AA-BB-CC-DD-EE-FF").as_deref(),
            Some("aa:bb:cc:dd:ee:ff")
        );
        let arp_a = "Interface: 192.168.1.10\n  192.168.1.1           aa-bb-cc-dd-ee-ff     dynamic\n";
        assert_eq!(
            parse_arp_a_for_ip(arp_a, "192.168.1.1").as_deref(),
            Some("aa:bb:cc:dd:ee:ff")
        );
        let proc = "IP address       HW type     Flags       HW address            Mask     Device\n\
192.168.1.1      0x1         0x2         aa:bb:cc:dd:ee:ff     *        eth0\n";
        assert_eq!(
            parse_proc_net_arp(proc, "192.168.1.1").as_deref(),
            Some("aa:bb:cc:dd:ee:ff")
        );
    }

    #[test]
    fn enrich_detects_arp_churn_and_cert_delta() {
        let mut current = base_report();
        current.environment.default_gateway = Some("192.168.1.1".to_string());
        let mut board = assess_integrity_board_with_parts(
            &current,
            Some(empty_listeners()),
            Some(GatewayArpSnapshot {
                gateway_ip: "192.168.1.1".to_string(),
                mac: Some("aa:bb:cc:dd:ee:ff".to_string()),
                source: "test".to_string(),
                observed: true,
            }),
            Some(CertStoreSnapshot {
                root_count: 60,
                fingerprint: "abcd:60".to_string(),
                source: "test".to_string(),
                observed: true,
            }),
        );
        assert!(!board.not_checked.iter().any(|s| s.contains("ARP")));
        assert!(!board.not_checked.iter().any(|s| s.contains("Trusted root")));

        let mut prior = base_report();
        prior.integrity = Some(IntegrityBoardReport {
            indicator_count: 0,
            level: IntegrityBoardLevel::None,
            title: "t".to_string(),
            summary: "s".to_string(),
            disclaimer: INTEGRITY_DISCLAIMER.to_string(),
            indicators: vec![],
            listeners: None,
            gateway_arp: Some(GatewayArpSnapshot {
                gateway_ip: "192.168.1.1".to_string(),
                mac: Some("11:22:33:44:55:66".to_string()),
                source: "test".to_string(),
                observed: true,
            }),
            cert_store: Some(CertStoreSnapshot {
                root_count: 55,
                fingerprint: "old:55".to_string(),
                source: "test".to_string(),
                observed: true,
            }),
            not_checked: vec![],
            confidence: IntegrityBoardConfidence::Low,
        });

        current.integrity = Some(board);
        enrich_integrity_board(&mut current, &[prior]);
        board = current.integrity.unwrap();
        assert!(board
            .indicators
            .iter()
            .any(|i| i.kind == IntegrityIndicatorKind::GatewayArpChurn));
        assert!(board
            .indicators
            .iter()
            .any(|i| i.kind == IntegrityIndicatorKind::CertStoreDelta));
        assert!(board.indicator_count >= 2);
        assert_eq!(board.level, IntegrityBoardLevel::Caution);
    }
}
