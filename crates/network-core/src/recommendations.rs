use crate::types::*;

pub fn build_recommendations(report: &HealthReport) -> NetworkRecommendations {
    let mut items = Vec::new();

    if let Some(context) = &report.network_context {
        push_context_recommendations(context, &report.environment, &mut items);
    }

    if let Some(egress) = &report.egress {
        push_egress_recommendations(egress, &mut items);
    }

    if let Some(integrity) = &report.dns_integrity {
        push_dns_recommendations(integrity, &mut items);
    }

    push_path_recommendations(report, &mut items);

    items.sort_by(|left, right| right.priority.cmp(&left.priority));

    let summary = if items.is_empty() {
        "No specific recommendations right now. Connection looks routine.".to_string()
    } else {
        format!("{} recommendation(s) based on current network context.", items.len())
    };

    NetworkRecommendations { items, summary }
}

fn push_context_recommendations(
    context: &NetworkContextReport,
    environment: &EnvironmentSnapshot,
    items: &mut Vec<NetworkRecommendation>,
) {
    let behind_vpn = environment.tags.contains(&EnvironmentTag::Vpn);

    match context.kind {
        NetworkContextKind::CaptivePortal => {
            items.push(NetworkRecommendation {
                category: RecommendationCategory::CaptivePortal,
                priority: 95,
                title: "Complete Wi-Fi sign-in".to_string(),
                message: "This network appears to require a login page before full internet access. Open your browser and finish the hotspot portal.".to_string(),
            });
        }
        NetworkContextKind::GuestWifi if !behind_vpn => {
            items.push(NetworkRecommendation {
                category: RecommendationCategory::PublicNetwork,
                priority: 85,
                title: "Treat guest Wi-Fi as untrusted".to_string(),
                message: "Hotel, café, and restaurant networks can inspect traffic. Avoid sensitive logins unless a VPN is active.".to_string(),
            });
        }
        NetworkContextKind::PublicCellular if !behind_vpn => {
            items.push(NetworkRecommendation {
                category: RecommendationCategory::PublicNetwork,
                priority: 80,
                title: "Cellular path is shared".to_string(),
                message: "Mobile data is generally safer than open Wi-Fi, but still use caution for banking and work accounts on unfamiliar routes.".to_string(),
            });
        }
        NetworkContextKind::GuestWifi | NetworkContextKind::PublicCellular if behind_vpn => {
            items.push(NetworkRecommendation {
                category: RecommendationCategory::VpnPrivacy,
                priority: 60,
                title: "VPN is protecting this path".to_string(),
                message: "A VPN tunnel is active. Keep it connected on guest networks and verify sites still load if performance drops.".to_string(),
            });
        }
        _ => {}
    }

    if matches!(context.risk_level, NetworkRiskLevel::High) {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::PublicNetwork,
            priority: 90,
            title: "High-risk network conditions".to_string(),
            message: context.summary.clone(),
        });
    }
}

fn push_egress_recommendations(egress: &EgressReport, items: &mut Vec<NetworkRecommendation>) {
    if let Some(ip) = &egress.primary_ip {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::Egress,
            priority: 40,
            title: "Public egress IP".to_string(),
            message: format!("Your current public IP appears to be {ip}."),
        });
    }

    if crate::egress::egress_unstable(egress) {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::Egress,
            priority: 70,
            title: "Unstable egress detection".to_string(),
            message: "IP providers disagreed or probes failed. Proxy, VPN, or captive portal interference may be present.".to_string(),
        });
    }

    if let (Some(system_ip), Some(tor)) = (&egress.primary_ip, egress.tor_path.as_ref()) {
        if let Some(tor_ip) = &tor.primary_ip {
            if system_ip != tor_ip {
                items.push(NetworkRecommendation {
                    category: RecommendationCategory::TorPath,
                    priority: 65,
                    title: "Tor egress differs from system path".to_string(),
                    message: format!(
                        "System path reports {system_ip} while Tor SOCKS reports {tor_ip}. Apps must route through Tor to use the anonymous egress."
                    ),
                });
            }
        }
    }
}

fn push_dns_recommendations(integrity: &DnsIntegrityStatus, items: &mut Vec<NetworkRecommendation>) {
    if matches!(
        integrity.state,
        DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
    ) {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::DnsSecurity,
            priority: 88,
            title: "DNS may be filtered or hijacked".to_string(),
            message: format!(
                "{} KnotTrace can apply trusted DNS automatically when DNS recovery is enabled.",
                integrity.summary
            ),
        });
    }
}

fn push_path_recommendations(report: &HealthReport, items: &mut Vec<NetworkRecommendation>) {
    let env = &report.environment;

    if env.proxy.enabled
        && report
            .site_reachability
            .as_ref()
            .is_some_and(crate::reachability::site_access_degraded)
    {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::ProxyPath,
            priority: 75,
            title: "Proxy path may be blocking sites".to_string(),
            message: "Verification sites failed while a proxy is active. Review Connect Assist manually — KnotTrace will not switch nodes automatically unless you enable it.".to_string(),
        });
    }

    if env.tor.detected && !env.tor.socks_reachable {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::TorPath,
            priority: 82,
            title: "Tor detected but SOCKS is unreachable".to_string(),
            message: "Tor appears configured but the local SOCKS port is not responding. Wait for bootstrap or check your Tor client.".to_string(),
        });
    }

    if env.tags.contains(&EnvironmentTag::Vpn)
        && matches!(report.score.grade, HealthGrade::Fair | HealthGrade::Poor)
    {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::VpnPrivacy,
            priority: 55,
            title: "VPN tunnel may be limiting performance".to_string(),
            message: "Latency or loss is elevated while a VPN interface is active. Try another server or split tunneling if your provider supports it.".to_string(),
        });
    }
}
