use crate::link_platform;
use crate::types::*;

const ETHERNET_CAPPED_MBPS: u32 = 100;

pub fn collect_link_facts(environment: &EnvironmentSnapshot) -> LinkFactsReport {
    let source = link_platform::platform_source().to_string();
    let adapters = match link_platform::enrich_adapters(&environment.interfaces) {
        Ok(adapters) => adapters,
        Err(_) => environment
            .interfaces
            .iter()
            .map(|iface| LinkAdapterFact {
                name: iface.name.clone(),
                friendly_name: iface.friendly_name.clone(),
                kind: iface.kind,
                is_up: iface.is_up,
                is_default_route: iface.is_default_route,
                speed_mbps: None,
                duplex: None,
                media: None,
                raw_speed: None,
            })
            .collect(),
    };

    build_link_facts_report(adapters, source)
}

pub fn build_link_facts_report(adapters: Vec<LinkAdapterFact>, source: String) -> LinkFactsReport {
    let active = adapters
        .iter()
        .find(|adapter| adapter.is_default_route)
        .cloned()
        .or_else(|| adapters.iter().find(|adapter| adapter.is_up).cloned());

    let issues = derive_issues(&adapters, active.as_ref());
    let summary = summarize(active.as_ref(), &issues);

    LinkFactsReport {
        active,
        adapters,
        issues,
        summary,
        source,
    }
}

fn derive_issues(adapters: &[LinkAdapterFact], active: Option<&LinkAdapterFact>) -> Vec<LinkIssue> {
    let mut issues = Vec::new();

    if let Some(active) = active {
        if active.kind == LinkKind::Ethernet {
            if let Some(speed) = active.speed_mbps {
                if speed > 0 && speed <= ETHERNET_CAPPED_MBPS {
                    issues.push(LinkIssue {
                        kind: LinkIssueKind::EthernetCapped,
                        severity: AlertLevel::Warning,
                        title: "Ethernet link looks capped".to_string(),
                        message: format!(
                            "Active Ethernet negotiated {speed} Mbps. A bad cable, switch port, or duplex mismatch can hold a gigabit NIC at 100 Mbps or less."
                        ),
                    });
                }
            }
            if matches!(active.duplex, Some(LinkDuplex::Half)) {
                issues.push(LinkIssue {
                    kind: LinkIssueKind::HalfDuplex,
                    severity: AlertLevel::Warning,
                    title: "Ethernet is half-duplex".to_string(),
                    message: "Half-duplex Ethernet often collapses throughput under load. Prefer auto-negotiation on both ends.".to_string(),
                });
            }
        }

        if active.kind == LinkKind::WiFi {
            let ethernet_up = adapters.iter().any(|adapter| {
                adapter.kind == LinkKind::Ethernet && adapter.is_up && !adapter.is_default_route
            });
            if ethernet_up {
                issues.push(LinkIssue {
                    kind: LinkIssueKind::PreferEthernet,
                    severity: AlertLevel::Warning,
                    title: "Wi-Fi is preferred while Ethernet is available".to_string(),
                    message: "An Ethernet adapter is up, but the default route still uses Wi-Fi. Prefer wired for large transfers and stable latency.".to_string(),
                });
            } else {
                issues.push(LinkIssue {
                    kind: LinkIssueKind::WifiActive,
                    severity: AlertLevel::Info,
                    title: "Active path uses Wi-Fi".to_string(),
                    message: match active.speed_mbps {
                        Some(speed) => format!(
                            "Wi-Fi link reports about {speed} Mbps. Real throughput is often lower; try Ethernet to compare."
                        ),
                        None => "Wi-Fi often caps real-world throughput below ISP plan speeds.".to_string(),
                    },
                });
            }
        }
    }

    issues
}

fn summarize(active: Option<&LinkAdapterFact>, issues: &[LinkIssue]) -> String {
    let Some(active) = active else {
        return "No active network adapter facts available.".to_string();
    };

    let label = active
        .friendly_name
        .as_deref()
        .unwrap_or(active.name.as_str());
    let kind = match active.kind {
        LinkKind::Ethernet => "Ethernet",
        LinkKind::WiFi => "Wi-Fi",
        LinkKind::Cellular => "cellular",
        LinkKind::Virtual => "virtual",
        LinkKind::Loopback => "loopback",
        LinkKind::Unknown => "unknown",
    };
    let speed = active
        .speed_mbps
        .map(|mbps| format!("{mbps} Mbps"))
        .or_else(|| active.raw_speed.clone())
        .unwrap_or_else(|| "speed unknown".to_string());

    if let Some(issue) = issues
        .iter()
        .find(|issue| matches!(issue.severity, AlertLevel::Warning | AlertLevel::Critical))
    {
        return format!("{kind} `{label}` · {speed}. {}", issue.title);
    }

    format!("Active {kind} `{label}` · {speed}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter(
        name: &str,
        kind: LinkKind,
        is_up: bool,
        is_default: bool,
        speed: Option<u32>,
        duplex: Option<LinkDuplex>,
    ) -> LinkAdapterFact {
        LinkAdapterFact {
            name: name.to_string(),
            friendly_name: Some(name.to_string()),
            kind,
            is_up,
            is_default_route: is_default,
            speed_mbps: speed,
            duplex,
            media: None,
            raw_speed: speed.map(|mbps| format!("{mbps} Mbps")),
        }
    }

    #[test]
    fn flags_capped_ethernet() {
        let report = build_link_facts_report(
            vec![adapter(
                "eth0",
                LinkKind::Ethernet,
                true,
                true,
                Some(100),
                Some(LinkDuplex::Full),
            )],
            "test".to_string(),
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.kind == LinkIssueKind::EthernetCapped)
        );
        assert!(report.summary.contains("capped") || report.summary.contains("100"));
    }

    #[test]
    fn prefers_ethernet_when_wifi_is_default() {
        let report = build_link_facts_report(
            vec![
                adapter("wlan0", LinkKind::WiFi, true, true, Some(400), None),
                adapter("eth0", LinkKind::Ethernet, true, false, Some(1000), Some(LinkDuplex::Full)),
            ],
            "test".to_string(),
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.kind == LinkIssueKind::PreferEthernet)
        );
    }

    #[test]
    fn gigabit_ethernet_is_clean() {
        let report = build_link_facts_report(
            vec![adapter(
                "eth0",
                LinkKind::Ethernet,
                true,
                true,
                Some(1000),
                Some(LinkDuplex::Full),
            )],
            "test".to_string(),
        );
        assert!(report.issues.is_empty());
    }
}
