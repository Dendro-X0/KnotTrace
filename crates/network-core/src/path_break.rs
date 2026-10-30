//! Path-break classifier (Path Truth Band 1).
//! Synthesizes existing HealthReport signals into path + hop + environment labels.

use crate::types::*;
use std::collections::HashMap;

const NOT_CHECKED: &[&str] = &[
    "Listening services / high-risk ports",
    "Trusted root certificate deltas",
    "Gateway / ARP identity churn",
    "Process ↔ egress correlation",
];

pub fn classify_path_break(report: &HealthReport) -> PathBreakReport {
    let paths_present = detect_paths_present(&report.environment);
    let active_path = choose_active_path(&report.environment, &paths_present);
    let (local_ok, local_issues) = assess_local(report);
    let environment_class = classify_environment(report, active_path);

    let (break_hop, confidence, mut evidence) = if !local_ok {
        classify_local_break(report, &local_issues)
    } else {
        classify_remote_break(report, active_path)
    };

    for issue in &local_issues {
        if !evidence.iter().any(|e| e == issue) {
            evidence.push(issue.clone());
        }
    }

    let title = build_title(local_ok, active_path, break_hop);
    let summary = build_summary(
        local_ok,
        active_path,
        break_hop,
        environment_class,
        &evidence,
    );

    PathBreakReport {
        local_ok,
        local_issues,
        active_path,
        paths_present,
        break_hop,
        environment_class,
        confidence,
        title,
        summary,
        evidence,
        not_checked: NOT_CHECKED.iter().map(|s| (*s).to_string()).collect(),
    }
}

fn detect_paths_present(environment: &EnvironmentSnapshot) -> Vec<PathBreakPath> {
    let mut paths = vec![PathBreakPath::Direct];
    if environment.proxy.enabled || environment.tags.contains(&EnvironmentTag::Proxy) {
        paths.push(PathBreakPath::SystemProxy);
    }
    if environment.tags.contains(&EnvironmentTag::Vpn) {
        paths.push(PathBreakPath::VpnTunnel);
    }
    if environment.tor.detected || environment.tags.contains(&EnvironmentTag::Tor) {
        paths.push(PathBreakPath::TorSocks);
    }
    paths
}

fn choose_active_path(
    environment: &EnvironmentSnapshot,
    paths_present: &[PathBreakPath],
) -> PathBreakPath {
    let tor_present = paths_present.contains(&PathBreakPath::TorSocks);
    if tor_present && environment.tor.socks_reachable {
        return PathBreakPath::TorSocks;
    }
    if tor_present && environment.tor.detected && !environment.tor.socks_reachable {
        // Still label Tor as active intent when SOCKS is down — hop will be Local.
        return PathBreakPath::TorSocks;
    }
    if paths_present.contains(&PathBreakPath::VpnTunnel) {
        return PathBreakPath::VpnTunnel;
    }
    if paths_present.contains(&PathBreakPath::SystemProxy) {
        return PathBreakPath::SystemProxy;
    }
    PathBreakPath::Direct
}

fn assess_local(report: &HealthReport) -> (bool, Vec<String>) {
    let mut issues = Vec::new();
    let env = &report.environment;

    if env.active_interface.is_none() && env.default_gateway.is_none() {
        issues.push("No active interface or default gateway detected.".to_string());
    }

    if let Some(gateway) = &report.probe.gateway {
        if gateway.loss_pct >= 100.0 {
            issues.push(format!(
                "Gateway probe to {} failed (100% loss).",
                gateway.target
            ));
        }
    } else if env.default_gateway.is_some() {
        // Missing sample is soft — do not fail local solely on absence.
    }

    let dns = &report.probe.dns;
    if !dns.is_empty() {
        let failed = dns.iter().filter(|d| !d.success).count();
        if failed * 2 >= dns.len() {
            issues.push(format!(
                "Local DNS probes failing ({failed}/{}).",
                dns.len()
            ));
        }
    }

    if let Some(context) = &report.network_context {
        match context.captive_portal.state {
            CaptivePortalState::Confirmed => {
                issues.push("Captive portal confirmed — local network gate before internet.".to_string());
            }
            CaptivePortalState::Suspected => {
                issues.push("Captive portal suspected — sign-in may be required.".to_string());
            }
            CaptivePortalState::NotDetected => {}
        }
    }

    if let Some(tor) = paths_tor_socks_down(report) {
        issues.push(tor);
    }

    let local_ok = issues.is_empty();
    (local_ok, issues)
}

fn paths_tor_socks_down(report: &HealthReport) -> Option<String> {
    let tor = &report.environment.tor;
    if (tor.detected || report.environment.tags.contains(&EnvironmentTag::Tor))
        && !tor.socks_reachable
    {
        Some(
            "Tor appears configured but SOCKS is unreachable (client/bootstrap), not a total internet outage."
                .to_string(),
        )
    } else {
        None
    }
}

fn classify_local_break(
    report: &HealthReport,
    local_issues: &[String],
) -> (PathBreakHop, PathBreakConfidence, Vec<String>) {
    let mut evidence: Vec<String> = local_issues.to_vec();
    let hop = if local_issues.iter().any(|i| i.contains("Captive portal")) {
        PathBreakHop::Local
    } else if local_issues.iter().any(|i| i.contains("SOCKS")) {
        PathBreakHop::Local
    } else if local_issues.iter().any(|i| i.contains("DNS")) {
        PathBreakHop::Dns
    } else if dns_integrity_suspicious(report) {
        evidence.push(report.dns_integrity.as_ref().unwrap().summary.clone());
        PathBreakHop::Dns
    } else {
        PathBreakHop::Local
    };
    (hop, PathBreakConfidence::High, evidence)
}

fn dns_integrity_suspicious(report: &HealthReport) -> bool {
    matches!(
        report.dns_integrity.as_ref().map(|d| d.state),
        Some(DnsIntegrityState::Suspicious) | Some(DnsIntegrityState::Caution)
    )
}

fn classify_remote_break(
    report: &HealthReport,
    active_path: PathBreakPath,
) -> (PathBreakHop, PathBreakConfidence, Vec<String>) {
    let mut evidence = Vec::new();

    if dns_integrity_suspicious(report) {
        if let Some(status) = &report.dns_integrity {
            evidence.push(status.summary.clone());
        }
        // Integrity alone is not a path break if reachability is fine — check failures first.
    }

    if let Some(compare) = &report.tunnel_compare {
        if let Some((hop, conf, mut notes)) = hop_from_tunnel_compare(compare, active_path) {
            evidence.append(&mut notes);
            return (hop, conf, evidence);
        }
    }

    if let Some(proxy_report) = &report.proxy_path_report {
        if proxy_report.proxy_only_failure_count > 0 {
            evidence.push(proxy_report.summary.clone());
            let hop = dominant_hop_from_results(
                proxy_report
                    .comparisons
                    .iter()
                    .filter(|c| c.proxy_only_failure)
                    .map(|c| &c.proxy),
            );
            return (
                hop.unwrap_or(PathBreakHop::Tcp),
                map_proxy_confidence(proxy_report.confidence),
                evidence,
            );
        }
    }

    if let Some(reach) = &report.site_reachability {
        if reach.failure_count > 0 {
            evidence.push(reach.summary.clone());
            let hop = dominant_hop_from_results(reach.results.iter())
                .unwrap_or(PathBreakHop::Unknown);
            let confidence = if reach.failure_count >= reach.success_count {
                PathBreakConfidence::High
            } else {
                PathBreakConfidence::Medium
            };
            if dns_integrity_suspicious(report) && hop == PathBreakHop::Unknown {
                return (PathBreakHop::Dns, confidence, evidence);
            }
            return (hop, confidence, evidence);
        }
    }

    if dns_integrity_suspicious(report) {
        return (PathBreakHop::Dns, PathBreakConfidence::Medium, evidence);
    }

    evidence.push("No path-break hop detected from current probes.".to_string());
    (PathBreakHop::None, PathBreakConfidence::Medium, evidence)
}

fn hop_from_tunnel_compare(
    compare: &TunnelPathCompareReport,
    active_path: PathBreakPath,
) -> Option<(PathBreakHop, PathBreakConfidence, Vec<String>)> {
    let kind = match active_path {
        PathBreakPath::Direct => TunnelPathKind::Direct,
        PathBreakPath::SystemProxy | PathBreakPath::VpnTunnel => TunnelPathKind::SystemProxy,
        PathBreakPath::TorSocks => TunnelPathKind::TorSocks,
    };

    let sample = compare.paths.iter().find(|p| p.kind == kind)?;
    if !sample.available {
        return Some((
            PathBreakHop::Local,
            PathBreakConfidence::High,
            vec![sample
                .note
                .clone()
                .unwrap_or_else(|| format!("{} path unavailable.", sample.label))],
        ));
    }
    if sample.failure_count == 0 {
        return None;
    }

    let hop = dominant_hop_from_results(sample.reachability.iter())
        .unwrap_or(PathBreakHop::Unknown);
    let mut notes = vec![format!(
        "{} path: {} success, {} failure.",
        sample.label, sample.success_count, sample.failure_count
    )];
    if let Some(note) = &sample.note {
        notes.push(note.clone());
    }
    let confidence = if sample.success_count == 0 {
        PathBreakConfidence::High
    } else {
        PathBreakConfidence::Medium
    };
    Some((hop, confidence, notes))
}

fn dominant_hop_from_results<'a, I>(results: I) -> Option<PathBreakHop>
where
    I: Iterator<Item = &'a SiteReachResult>,
{
    let mut counts: HashMap<PathBreakHop, u32> = HashMap::new();
    for result in results {
        if result.success {
            continue;
        }
        let hop = match result.error_kind {
            Some(SiteReachErrorKind::Dns) => PathBreakHop::Dns,
            Some(SiteReachErrorKind::Timeout) | Some(SiteReachErrorKind::ConnectionReset) => {
                PathBreakHop::Tcp
            }
            Some(SiteReachErrorKind::Tls) => PathBreakHop::Tls,
            Some(SiteReachErrorKind::HttpServer) | Some(SiteReachErrorKind::HttpBlocked) => {
                PathBreakHop::Http
            }
            Some(SiteReachErrorKind::Proxy) => PathBreakHop::Tcp,
            Some(SiteReachErrorKind::Unknown) | None => PathBreakHop::Unknown,
        };
        *counts.entry(hop).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(hop, _)| hop)
}

fn map_proxy_confidence(confidence: ProxyPathConfidence) -> PathBreakConfidence {
    match confidence {
        ProxyPathConfidence::High => PathBreakConfidence::High,
        ProxyPathConfidence::Medium => PathBreakConfidence::Medium,
        ProxyPathConfidence::Low => PathBreakConfidence::Low,
    }
}

fn classify_environment(
    report: &HealthReport,
    active_path: PathBreakPath,
) -> PathEnvironmentClass {
    if let Some(context) = &report.network_context {
        match context.captive_portal.state {
            CaptivePortalState::Confirmed | CaptivePortalState::Suspected => {
                return PathEnvironmentClass::CaptivePortal;
            }
            CaptivePortalState::NotDetected => {}
        }
        match context.kind {
            NetworkContextKind::CaptivePortal => return PathEnvironmentClass::CaptivePortal,
            NetworkContextKind::GuestWifi => return PathEnvironmentClass::GuestWifi,
            NetworkContextKind::PublicCellular => return PathEnvironmentClass::PublicHotspot,
            NetworkContextKind::HomeLan => {
                if active_path == PathBreakPath::Direct && !report.environment.proxy.enabled {
                    return PathEnvironmentClass::HomeLan;
                }
            }
            NetworkContextKind::Unknown => {}
        }
    }

    match active_path {
        PathBreakPath::TorSocks => PathEnvironmentClass::Tor,
        PathBreakPath::VpnTunnel => PathEnvironmentClass::VpnOnly,
        PathBreakPath::SystemProxy => PathEnvironmentClass::CorporateProxy,
        PathBreakPath::Direct => {
            if report.environment.tags.contains(&EnvironmentTag::Public) {
                PathEnvironmentClass::PublicHotspot
            } else if report.environment.tags.contains(&EnvironmentTag::HomeLan) {
                PathEnvironmentClass::HomeLan
            } else {
                PathEnvironmentClass::Unknown
            }
        }
    }
}

fn path_label(path: PathBreakPath) -> &'static str {
    match path {
        PathBreakPath::Direct => "Direct",
        PathBreakPath::SystemProxy => "System proxy",
        PathBreakPath::VpnTunnel => "VPN",
        PathBreakPath::TorSocks => "Tor SOCKS",
    }
}

fn hop_label(hop: PathBreakHop) -> &'static str {
    match hop {
        PathBreakHop::None => "no break",
        PathBreakHop::Local => "local",
        PathBreakHop::Dns => "DNS",
        PathBreakHop::Tcp => "TCP",
        PathBreakHop::Tls => "TLS",
        PathBreakHop::Http => "HTTP",
        PathBreakHop::Unknown => "unknown hop",
    }
}

fn build_title(local_ok: bool, active_path: PathBreakPath, break_hop: PathBreakHop) -> String {
    if break_hop == PathBreakHop::None {
        return format!("{} path looks reachable", path_label(active_path));
    }
    if !local_ok {
        return format!("Local path issue before {} egress", path_label(active_path));
    }
    format!(
        "{} path fails at {}",
        path_label(active_path),
        hop_label(break_hop)
    )
}

fn build_summary(
    local_ok: bool,
    active_path: PathBreakPath,
    break_hop: PathBreakHop,
    environment_class: PathEnvironmentClass,
    evidence: &[String],
) -> String {
    let env = match environment_class {
        PathEnvironmentClass::HomeLan => "home LAN",
        PathEnvironmentClass::GuestWifi => "guest Wi‑Fi",
        PathEnvironmentClass::PublicHotspot => "public/hotspot",
        PathEnvironmentClass::CaptivePortal => "captive portal",
        PathEnvironmentClass::CorporateProxy => "proxy / corporate-like",
        PathEnvironmentClass::VpnOnly => "VPN",
        PathEnvironmentClass::Tor => "Tor",
        PathEnvironmentClass::FilteredRegion => "filtered-region class",
        PathEnvironmentClass::Unknown => "unclassified environment",
    };

    let head = if break_hop == PathBreakHop::None {
        format!(
            "Active path: {} in a {} setting. No hop failure classified from current probes.",
            path_label(active_path),
            env
        )
    } else if !local_ok {
        format!(
            "Local checks failed before blaming {} ({}). Fix local gate first.",
            path_label(active_path),
            hop_label(break_hop)
        )
    } else {
        format!(
            "Active path: {} in a {} setting. Break classified at {}.",
            path_label(active_path),
            env,
            hop_label(break_hop)
        )
    };

    if let Some(first) = evidence.first() {
        format!("{head} Evidence: {first}")
    } else {
        head
    }
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
                interfaces: vec![NetworkInterface {
                    name: "eth0".to_string(),
                    friendly_name: Some("Ethernet".to_string()),
                    kind: LinkKind::Ethernet,
                    is_up: true,
                    ipv4: vec!["192.168.1.10".to_string()],
                    gateway: Some("192.168.1.1".to_string()),
                    is_default_route: true,
                }],
                dns_servers: vec![DnsServer {
                    address: "1.1.1.1".to_string(),
                }],
                proxy: ProxySettings {
                    enabled: false,
                    server: None,
                    source: "none".to_string(),
                },
                tags: vec![EnvironmentTag::HomeLan],
                default_gateway: Some("192.168.1.1".to_string()),
                active_interface: Some("eth0".to_string()),
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: Some(LatencySample {
                    target: "192.168.1.1".to_string(),
                    avg_ms: 1.0,
                    loss_pct: 0.0,
                }),
                internet: Some(LatencySample {
                    target: "1.1.1.1".to_string(),
                    avg_ms: 20.0,
                    loss_pct: 0.0,
                }),
                dns: vec![DnsProbe {
                    resolver: "1.1.1.1".to_string(),
                    query: "example.com".to_string(),
                    latency_ms: 15.0,
                    success: true,
                }],
                duration_ms: 10,
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
            site_reachability: Some(SiteReachabilityStatus {
                checked_domains: 2,
                success_count: 2,
                failure_count: 0,
                results: vec![],
                summary: "All sites reachable.".to_string(),
            }),
            egress: None,
            network_context: Some(NetworkContextReport {
                kind: NetworkContextKind::HomeLan,
                risk_level: NetworkRiskLevel::Low,
                captive_portal: CaptivePortalStatus {
                    state: CaptivePortalState::NotDetected,
                    probe_url: "http://connectivitycheck.gstatic.com/generate_204".to_string(),
                    status_code: Some(204),
                    redirected: false,
                    summary: "No captive portal detected.".to_string(),
                },
                signals: Vec::new(),
                summary: "Home LAN".to_string(),
            }),
            recommendations: None,
            proxy_path_report: None,
            link_facts: None,
            local_caps: None,
            mtu_assist: None,
            local_repair: None,
            tunnel_compare: None,
            upstream_pool: None,
            path_break: None,
        }
    }

    #[test]
    fn healthy_direct_home() {
        let report = base_report();
        let path = classify_path_break(&report);
        assert!(path.local_ok);
        assert_eq!(path.active_path, PathBreakPath::Direct);
        assert_eq!(path.break_hop, PathBreakHop::None);
        assert_eq!(path.environment_class, PathEnvironmentClass::HomeLan);
        assert!(!path.not_checked.is_empty());
    }

    #[test]
    fn local_dns_failure_not_blamed_on_vpn() {
        let mut report = base_report();
        report.environment.tags.push(EnvironmentTag::Vpn);
        report.probe.dns = vec![
            DnsProbe {
                resolver: "system".to_string(),
                query: "a".to_string(),
                latency_ms: 0.0,
                success: false,
            },
            DnsProbe {
                resolver: "system".to_string(),
                query: "b".to_string(),
                latency_ms: 0.0,
                success: false,
            },
        ];
        let path = classify_path_break(&report);
        assert!(!path.local_ok);
        assert_eq!(path.active_path, PathBreakPath::VpnTunnel);
        assert_eq!(path.break_hop, PathBreakHop::Dns);
        assert!(path.title.contains("Local"));
    }

    #[test]
    fn tor_socks_down_is_local_hop() {
        let mut report = base_report();
        report.environment.tor = TorStatus {
            detected: true,
            socks_endpoint: Some("127.0.0.1:9050".to_string()),
            socks_reachable: false,
            source: "test".to_string(),
        };
        report.environment.tags.push(EnvironmentTag::Tor);
        let path = classify_path_break(&report);
        assert!(!path.local_ok);
        assert_eq!(path.active_path, PathBreakPath::TorSocks);
        assert_eq!(path.break_hop, PathBreakHop::Local);
        assert_eq!(path.environment_class, PathEnvironmentClass::Tor);
    }

    #[test]
    fn proxy_tls_failures_classify_tls_hop() {
        let mut report = base_report();
        report.environment.proxy.enabled = true;
        report.environment.proxy.server = Some("127.0.0.1:7890".to_string());
        report.environment.tags.push(EnvironmentTag::Proxy);
        report.site_reachability = Some(SiteReachabilityStatus {
            checked_domains: 2,
            success_count: 0,
            failure_count: 2,
            results: vec![
                SiteReachResult {
                    domain: "github.com".to_string(),
                    success: false,
                    status_code: None,
                    latency_ms: None,
                    error: Some("tls".to_string()),
                    error_kind: Some(SiteReachErrorKind::Tls),
                },
                SiteReachResult {
                    domain: "www.cloudflare.com".to_string(),
                    success: false,
                    status_code: None,
                    latency_ms: None,
                    error: Some("tls".to_string()),
                    error_kind: Some(SiteReachErrorKind::Tls),
                },
            ],
            summary: "TLS failures on proxy path.".to_string(),
        });
        let path = classify_path_break(&report);
        assert!(path.local_ok);
        assert_eq!(path.active_path, PathBreakPath::SystemProxy);
        assert_eq!(path.break_hop, PathBreakHop::Tls);
        assert!(path.title.contains("TLS"));
    }
}
