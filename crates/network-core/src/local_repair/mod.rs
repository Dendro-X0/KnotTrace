//! Opt-in local repair actions that work without a working internet path.

#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod stub;

#[cfg(windows)]
use windows as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
use stub as platform;

use crate::types::*;

pub fn collect_local_repair(report: &HealthReport) -> LocalRepairReport {
    let available = platform::platform_available();
    let gateway_bad = report.probe.gateway.as_ref().is_some_and(|gateway| {
        gateway.loss_pct >= 50.0 || gateway.avg_ms >= 80.0
    }) || report.probe.gateway.is_none();
    let internet_down = report
        .probe
        .internet
        .as_ref()
        .is_none_or(|sample| sample.loss_pct >= 50.0);
    let dns_bad = report.dns_integrity.as_ref().is_some_and(|integrity| {
        !matches!(integrity.state, DnsIntegrityState::Ok)
    }) || report.probe.dns.iter().any(|sample| !sample.success);
    let weak_wifi = report.link_facts.as_ref().is_some_and(|facts| {
        facts
            .issues
            .iter()
            .any(|issue| matches!(issue.kind, LinkIssueKind::WifiWeakSignal))
    });
    let no_ipv4 = report
        .environment
        .interfaces
        .iter()
        .find(|iface| iface.is_default_route || iface.is_up)
        .is_some_and(|iface| iface.ipv4.is_empty());

    let local_problem_likely = gateway_bad || dns_bad || no_ipv4 || (internet_down && gateway_bad);

    let actions = vec![
        LocalRepairAction {
            kind: LocalRepairActionKind::FlushDns,
            title: "Flush DNS cache".to_string(),
            message: if dns_bad {
                "DNS integrity or lookups look unhealthy — clearing the resolver cache is a safe first step.".to_string()
            } else {
                "Clear stale DNS answers. Harmless if nothing is wrong.".to_string()
            },
            recommended: dns_bad,
            available,
        },
        LocalRepairAction {
            kind: LocalRepairActionKind::RenewDhcp,
            title: "Renew DHCP lease".to_string(),
            message: if gateway_bad || no_ipv4 {
                "Gateway or IPv4 looks broken — renewing DHCP can restore address/DNS from the router.".to_string()
            } else {
                "Rebind IPv4 from the LAN DHCP server. May briefly interrupt connectivity.".to_string()
            },
            recommended: gateway_bad || no_ipv4,
            available: available && platform::supports_dhcp_renew(),
        },
        LocalRepairAction {
            kind: LocalRepairActionKind::RestartAdapter,
            title: "Restart active adapter".to_string(),
            message: if weak_wifi || gateway_bad {
                "Disable/enable the active NIC. Helps stuck Wi‑Fi/link state; needs admin on many systems.".to_string()
            } else {
                "Cycle the active network adapter. More disruptive — use when the link looks wedged.".to_string()
            },
            recommended: gateway_bad || weak_wifi,
            available: available && platform::supports_adapter_restart(),
        },
    ];

    let summary = if !available {
        platform::platform_note()
    } else if local_problem_likely {
        "Local connectivity looks impaired — try the recommended repair actions below (not ISP/CDN fixes)."
            .to_string()
    } else if internet_down {
        "Internet probes failed but the local gateway may still be OK. Local repair won't fix an ISP or Steam CDN outage."
            .to_string()
    } else {
        "No strong local-repair signal right now. Actions below remain available if you want a classic reset."
            .to_string()
    };

    LocalRepairReport {
        available,
        platform_note: platform::platform_note(),
        actions,
        summary,
        local_problem_likely,
    }
}

pub fn run_local_repair_action(
    kind: LocalRepairActionKind,
    environment: &EnvironmentSnapshot,
) -> Result<LocalRepairResult, String> {
    if !platform::platform_available() {
        return Err(platform::platform_note());
    }

    let message = match kind {
        LocalRepairActionKind::FlushDns => platform::flush_dns()?,
        LocalRepairActionKind::RenewDhcp => {
            let alias = resolve_alias(environment)
                .ok_or_else(|| "No active adapter found for DHCP renew.".to_string())?;
            platform::renew_dhcp(&alias)?
        }
        LocalRepairActionKind::RestartAdapter => {
            let alias = resolve_alias(environment)
                .ok_or_else(|| "No active adapter found to restart.".to_string())?;
            platform::restart_adapter(&alias)?
        }
    };

    Ok(LocalRepairResult {
        kind,
        success: true,
        message,
    })
}

fn resolve_alias(environment: &EnvironmentSnapshot) -> Option<String> {
    environment
        .interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| environment.interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| {
            iface
                .friendly_name
                .clone()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| iface.name.clone())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_report(dns_bad: bool, gateway_ok: bool) -> HealthReport {
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
                dns_servers: Vec::new(),
                proxy: ProxySettings {
                    enabled: false,
                    server: None,
                    source: "none".to_string(),
                },
                tags: Vec::new(),
                default_gateway: Some("192.168.1.1".to_string()),
                active_interface: Some("Ethernet".to_string()),
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: if gateway_ok {
                    Some(LatencySample {
                        target: "gateway".to_string(),
                        avg_ms: 2.0,
                        loss_pct: 0.0,
                    })
                } else {
                    None
                },
                internet: None,
                dns: Vec::new(),
                duration_ms: 1,
            },
            score: HealthScore {
                score: 40,
                grade: HealthGrade::Poor,
                summary: "test".to_string(),
                reasons: Vec::new(),
            },
            dns_integrity: if dns_bad {
                Some(DnsIntegrityStatus {
                    state: DnsIntegrityState::Suspicious,
                    confidence: DnsIntegrityConfidence::Medium,
                    summary: "bad".to_string(),
                    checked_domains: 2,
                    mismatch_count: 1,
                    details: Vec::new(),
                })
            } else {
                None
            },
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

    #[test]
    fn recommends_flush_when_dns_suspicious() {
        let report = collect_local_repair(&sample_report(true, true));
        let flush = report
            .actions
            .iter()
            .find(|action| matches!(action.kind, LocalRepairActionKind::FlushDns))
            .expect("flush action");
        assert!(flush.recommended);
    }

    #[test]
    fn recommends_dhcp_when_gateway_missing() {
        let report = collect_local_repair(&sample_report(false, false));
        let dhcp = report
            .actions
            .iter()
            .find(|action| matches!(action.kind, LocalRepairActionKind::RenewDhcp))
            .expect("dhcp action");
        assert!(dhcp.recommended);
        assert!(report.local_problem_likely);
    }
}
