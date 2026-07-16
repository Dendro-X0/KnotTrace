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
    push_link_recommendations(report, &mut items);
    push_local_caps_recommendations(report, &mut items);
    push_mtu_assist_recommendations(report, &mut items);
    push_tunnel_compare_recommendations(report, &mut items);
    push_upstream_pool_recommendations(report, &mut items);
    boost_recommendations_for_shape(report, &mut items);

    items.sort_by(|left, right| right.priority.cmp(&left.priority));

    let summary = if items.is_empty() {
        "No specific recommendations right now. Connection looks routine.".to_string()
    } else {
        format!("{} recommendation(s) based on current network context.", items.len())
    };

    NetworkRecommendations { items, summary }
}

fn boost_recommendations_for_shape(report: &HealthReport, items: &mut [NetworkRecommendation]) {
    let Some(diagnosis) = &report.diagnosis else {
        return;
    };

    for item in items.iter_mut() {
        let bump = match diagnosis.slowdown_shape {
            SlowdownShape::PageStart => match item.category {
                RecommendationCategory::DnsSecurity | RecommendationCategory::CaptivePortal => 8,
                RecommendationCategory::ProxyPath => 4,
                _ => 0,
            },
            SlowdownShape::UnderLoadLag => match item.category {
                RecommendationCategory::General => 4,
                RecommendationCategory::Egress => -2,
                _ => 0,
            },
            SlowdownShape::PartialSiteFailure => match item.category {
                RecommendationCategory::ProxyPath | RecommendationCategory::TorPath => 8,
                RecommendationCategory::DnsSecurity => 4,
                _ => 0,
            },
            SlowdownShape::RestrictedNetwork => match item.category {
                RecommendationCategory::CaptivePortal | RecommendationCategory::PublicNetwork => 10,
                RecommendationCategory::VpnPrivacy => 5,
                _ => 0,
            },
            SlowdownShape::TunnelOverhead => match item.category {
                RecommendationCategory::VpnPrivacy | RecommendationCategory::TorPath => 8,
                RecommendationCategory::ProxyPath => 4,
                _ => 0,
            },
            SlowdownShape::LinkLocalIssue => match item.category {
                RecommendationCategory::LinkLocal | RecommendationCategory::General => 10,
                RecommendationCategory::PublicNetwork => -4,
                RecommendationCategory::Egress => -4,
                _ => 0,
            },
            SlowdownShape::GeneralDegradation => 0,
        };

        item.priority = (item.priority as i16 + bump).clamp(0, 100) as u8;
    }
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

    if env.proxy.enabled {
        if let Some(path) = &report.proxy_path_report {
            if path.likely_provider_side || path.proxy_failure_count > 0 {
                let thrash_forbidden = report.upstream_pool.as_ref().is_some_and(|proof| {
                    matches!(
                        proof.claim,
                        UpstreamPoolClaim::UpstreamPoolPoor
                            | UpstreamPoolClaim::ActivePathRecurring
                    )
                });
                let priority = match path.confidence {
                    ProxyPathConfidence::High => 82,
                    ProxyPathConfidence::Medium => 78,
                    ProxyPathConfidence::Low => 70,
                };
                items.push(NetworkRecommendation {
                    category: RecommendationCategory::ProxyPath,
                    priority,
                    title: if path.likely_provider_side {
                        "Proxy provider path is impairing access".to_string()
                    } else {
                        "Proxy path comparison found failures".to_string()
                    },
                    message: if thrash_forbidden {
                        format!(
                            "{} Do not rapidly switch nodes — upstream pool evidence says thrashing will not help.",
                            path.summary
                        )
                    } else {
                        format!(
                            "{} Review Upstream pool proof. One careful node change is optional; KnotTrace will not auto-rotate exits.",
                            path.summary
                        )
                    },
                });
            }
        } else if report
            .site_reachability
            .as_ref()
            .is_some_and(crate::reachability::site_access_degraded)
        {
            let failing_proxy_sites = report
                .site_reachability
                .as_ref()
                .map(|reachability| crate::reachability::proxy_verification_failures(reachability))
                .unwrap_or_default();

            let (title, message, priority) = if !failing_proxy_sites.is_empty() {
                let sample = failing_proxy_sites
                    .iter()
                    .take(2)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    "Proxy path is breaking major services".to_string(),
                    format!(
                        "High-signal sites failed while a proxy is active (example: {sample}). This strongly suggests the current proxy node is unstable, filtered, or resetting connections. Review Connect Assist manually — KnotTrace will not switch nodes automatically unless you enable it."
                    ),
                    85,
                )
            } else {
                (
                    "Proxy path may be blocking sites".to_string(),
                    "Verification sites failed while a proxy is active. Review Connect Assist manually — KnotTrace will not switch nodes automatically unless you enable it.".to_string(),
                    75,
                )
            };

            items.push(NetworkRecommendation {
                category: RecommendationCategory::ProxyPath,
                priority,
                title,
                message,
            });
        }
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

fn push_link_recommendations(report: &HealthReport, items: &mut Vec<NetworkRecommendation>) {
    let Some(facts) = &report.link_facts else {
        return;
    };

    for issue in &facts.issues {
        let (priority, title, message) = match issue.kind {
            LinkIssueKind::EthernetCapped => (
                78,
                "Check Ethernet cable or switch port".to_string(),
                issue.message.clone(),
            ),
            LinkIssueKind::HalfDuplex => (
                76,
                "Fix Ethernet duplex mismatch".to_string(),
                issue.message.clone(),
            ),
            LinkIssueKind::PreferEthernet => (
                72,
                "Prefer Ethernet over Wi-Fi".to_string(),
                issue.message.clone(),
            ),
            LinkIssueKind::WifiActive => (
                40,
                "Wi-Fi may limit throughput".to_string(),
                issue.message.clone(),
            ),
        };
        items.push(NetworkRecommendation {
            category: RecommendationCategory::LinkLocal,
            priority,
            title,
            message,
        });
    }
}

fn push_upstream_pool_recommendations(report: &HealthReport, items: &mut Vec<NetworkRecommendation>) {
    let Some(proof) = &report.upstream_pool else {
        return;
    };
    if matches!(proof.claim, UpstreamPoolClaim::None) {
        return;
    }

    let priority = match proof.claim {
        UpstreamPoolClaim::UpstreamPoolPoor => 92,
        UpstreamPoolClaim::ActivePathRecurring => 86,
        UpstreamPoolClaim::ActivePathImpaired => 80,
        UpstreamPoolClaim::Inconclusive => 35,
        UpstreamPoolClaim::None => return,
    };

    items.push(NetworkRecommendation {
        category: RecommendationCategory::ProxyPath,
        priority,
        title: proof.title.clone(),
        message: format!("{} {}", proof.summary, proof.action),
    });
}

fn push_tunnel_compare_recommendations(report: &HealthReport, items: &mut Vec<NetworkRecommendation>) {
    let Some(compare) = &report.tunnel_compare else {
        return;
    };

    if compare.tor_detected && !compare.tor_socks_reachable {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::TorPath,
            priority: 88,
            title: "Fix Tor SOCKS before blaming your ISP".to_string(),
            message: compare.expectation.clone(),
        });
        return;
    }

    if compare.tor_detected {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::TorPath,
            priority: 50,
            title: "Tor path is expected to be slower".to_string(),
            message: format!(
                "{} Open Network → Tunnel compare for Direct vs Tor samples. KnotTrace will not accelerate Tor.",
                compare.summary
            ),
        });
    }

    if compare.vpn_detected {
        items.push(NetworkRecommendation {
            category: RecommendationCategory::VpnPrivacy,
            priority: 48,
            title: "VPN overhead is normal".to_string(),
            message: compare.expectation.clone(),
        });
    }
}

fn push_local_caps_recommendations(report: &HealthReport, items: &mut Vec<NetworkRecommendation>) {
    let Some(caps) = &report.local_caps else {
        return;
    };
    if !caps.available || caps.issues.is_empty() {
        return;
    }

    let priority = if caps
        .issues
        .iter()
        .any(|issue| matches!(issue.severity, AlertLevel::Warning | AlertLevel::Critical))
    {
        74
    } else {
        45
    };

    items.push(NetworkRecommendation {
        category: RecommendationCategory::LinkLocal,
        priority,
        title: if caps.can_repair {
            "Repair Windows local caps (opt-in)".to_string()
        } else if caps.repair_active {
            "Windows local caps repair is active".to_string()
        } else {
            "Review Windows TCP / NIC power settings".to_string()
        },
        message: format!(
            "{} Use the Network page Local caps panel for a reversible repair — KnotTrace will not change these settings automatically.",
            caps.summary
        ),
    });
}

fn push_mtu_assist_recommendations(report: &HealthReport, items: &mut Vec<NetworkRecommendation>) {
    let Some(assist) = &report.mtu_assist else {
        return;
    };
    if !assist.fragmentation_risk {
        return;
    }

    let priority = if assist.can_repair {
        76
    } else if assist.repair_active {
        55
    } else if assist.tunnel_evidenced {
        48
    } else {
        30
    };

    items.push(NetworkRecommendation {
        category: RecommendationCategory::LinkLocal,
        priority,
        title: if assist.can_repair {
            "Opt-in MTU clamp for tunnel path".to_string()
        } else if assist.repair_active {
            "MTU assist clamp is active".to_string()
        } else {
            "Review path MTU / fragmentation risk".to_string()
        },
        message: format!(
            "{} Open Network → MTU assist. KnotTrace never changes MTU automatically.",
            assist.summary
        ),
    });
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
                interfaces: Vec::new(),
                dns_servers: Vec::new(),
                proxy: ProxySettings {
                    enabled: true,
                    server: Some("127.0.0.1:7890".to_string()),
                    source: "test".to_string(),
                },
                tags: Vec::new(),
                default_gateway: None,
                active_interface: None,
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: Some(LatencySample {
                    target: "internet".to_string(),
                    avg_ms: 40.0,
                    loss_pct: 0.0,
                }),
                dns: Vec::new(),
                duration_ms: 0,
            },
            score: HealthScore {
                grade: HealthGrade::Fair,
                score: 60,
                summary: "test".to_string(),
                reasons: Vec::new(),
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
            site_reachability: Some(SiteReachabilityStatus {
                checked_domains: 4,
                success_count: 1,
                failure_count: 3,
                results: vec![
                    SiteReachResult {
                        domain: "www.google.com".to_string(),
                        success: false,
                        status_code: None,
                        latency_ms: Some(100.0),
                        error: Some("reset".to_string()),
                        error_kind: Some(SiteReachErrorKind::ConnectionReset),
                    },
                    SiteReachResult {
                        domain: "example.com".to_string(),
                        success: true,
                        status_code: Some(200),
                        latency_ms: Some(50.0),
                        error: None,
                        error_kind: None,
                    },
                ],
                summary: "failed".to_string(),
            }),
            egress: None,
            network_context: None,
            recommendations: None,
            proxy_path_report: None,
            link_facts: None,
            local_caps: None,
            mtu_assist: None,
            tunnel_compare: None,
            upstream_pool: None,
        }
    }

    #[test]
    fn escalates_proxy_recommendation_when_major_sites_fail() {
        let report = base_report();
        let recs = build_recommendations(&report);
        let item = recs
            .items
            .iter()
            .find(|item| item.category == RecommendationCategory::ProxyPath)
            .expect("proxy recommendation should exist");
        assert!(item.title.to_ascii_lowercase().contains("major services"));
        assert!(item.message.to_ascii_lowercase().contains("high-signal"));
        assert!(item.priority >= 80);
    }
}
