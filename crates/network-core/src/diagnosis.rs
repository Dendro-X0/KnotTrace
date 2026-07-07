use crate::types::*;

const DNS_SLOW_MS: f64 = 80.0;
const INTERNET_HIGH_LATENCY_MS: f64 = 100.0;
const GATEWAY_HIGH_LATENCY_MS: f64 = 20.0;

pub fn diagnose_network(report: &HealthReport) -> NetworkDiagnosis {
    let mut hints = Vec::new();

    push_internet_hints(report, &mut hints);
    push_gateway_hints(report, &mut hints);
    push_dns_hints(report, &mut hints);
    push_integrity_hints(report, &mut hints);
    push_reachability_hints(report, &mut hints);
    push_context_hints(report, &mut hints);
    push_egress_hints(report, &mut hints);
    push_stability_hints(report, &mut hints);
    push_path_hints(report, &mut hints);

    if hints.is_empty() {
        return NetworkDiagnosis {
            summary: "No obvious bottlenecks detected from current probes.".to_string(),
            primary_bottleneck: Some(BottleneckCategory::Healthy),
            slowdown_shape: SlowdownShape::GeneralDegradation,
            confidence: DiagnosisConfidence::Low,
            hints: vec![healthy_hint()],
        };
    }

    let shape = classify_slowdown_shape(report);
    let confidence = diagnosis_confidence(report, shape);
    hints.sort_by(|left, right| {
        hint_priority(report, shape, right)
            .cmp(&hint_priority(report, shape, left))
            .then_with(|| severity_rank(right.severity).cmp(&severity_rank(left.severity)))
    });

    let primary_bottleneck = hints.first().map(|hint| hint.category);
    let summary = build_summary(primary_bottleneck, hints.len(), shape, confidence);

    NetworkDiagnosis {
        summary,
        primary_bottleneck,
        slowdown_shape: shape,
        confidence,
        hints,
    }
}

fn healthy_hint() -> BottleneckHint {
    BottleneckHint {
        category: BottleneckCategory::Healthy,
        severity: AlertLevel::Info,
        title: "Path looks responsive".to_string(),
        message: "Latency and DNS probes look normal. Throughput limits may still come from Wi-Fi, VPN, or ISP caps.".to_string(),
        suggestions: vec![
            "Use a benchmark snapshot before and after assists to compare.".to_string(),
            "Run Speedtest.net on Ethernet if you need Mbps numbers.".to_string(),
        ],
    }
}

fn push_internet_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let Some(internet) = &report.probe.internet else {
        hints.push(BottleneckHint {
            category: BottleneckCategory::InternetUnreachable,
            severity: AlertLevel::Critical,
            title: "Internet path unreachable".to_string(),
            message: "TCP probes to a public endpoint failed.".to_string(),
            suggestions: vec![
                "Check modem, router, and VPN/proxy status.".to_string(),
                "Try Ethernet instead of Wi-Fi.".to_string(),
            ],
        });
        return;
    };

    if internet.loss_pct >= 50.0 {
        hints.push(BottleneckHint {
            category: BottleneckCategory::InternetLoss,
            severity: AlertLevel::Critical,
            title: "High internet packet loss".to_string(),
            message: format!("{:.0}% of internet connection attempts failed.", internet.loss_pct),
            suggestions: vec![
                "Inspect Wi-Fi signal, cables, and router load.".to_string(),
                "Disable VPN/proxy briefly to isolate the path.".to_string(),
            ],
        });
    } else if internet.loss_pct >= 15.0 {
        hints.push(BottleneckHint {
            category: BottleneckCategory::InternetLoss,
            severity: AlertLevel::Warning,
            title: "Unstable internet path".to_string(),
            message: format!("{:.0}% connection loss to the internet probe.", internet.loss_pct),
            suggestions: vec!["Retry on Ethernet. Check for bufferbloat on your router.".to_string()],
        });
    }

    if internet.avg_ms >= INTERNET_HIGH_LATENCY_MS {
        hints.push(BottleneckHint {
            category: BottleneckCategory::InternetLatency,
            severity: AlertLevel::Warning,
            title: "High internet latency".to_string(),
            message: format!("Average internet RTT is {:.0} ms.", internet.avg_ms),
            suggestions: vec![
                "Compare with VPN/proxy off.".to_string(),
                "Try a different proxy node in Connect Assist.".to_string(),
            ],
        });
    }
}

fn push_gateway_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let Some(gateway) = &report.probe.gateway else {
        return;
    };

    if gateway.loss_pct >= 50.0 {
        hints.push(BottleneckHint {
            category: BottleneckCategory::Gateway,
            severity: AlertLevel::Warning,
            title: "Local gateway unstable".to_string(),
            message: "Your router or local link is dropping probes.".to_string(),
            suggestions: vec![
                "Move closer to Wi-Fi or use Ethernet.".to_string(),
                "Reboot the router if loss persists.".to_string(),
            ],
        });
    } else if gateway.avg_ms >= GATEWAY_HIGH_LATENCY_MS {
        hints.push(BottleneckHint {
            category: BottleneckCategory::Gateway,
            severity: AlertLevel::Info,
            title: "Elevated gateway latency".to_string(),
            message: format!("Local gateway RTT is {:.0} ms.", gateway.avg_ms),
            suggestions: vec!["Wi-Fi congestion or mesh hops may be the cause.".to_string()],
        });
    }
}

fn push_dns_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let failures = report.probe.dns.iter().filter(|sample| !sample.success).count();
    if failures > 0 {
        hints.push(BottleneckHint {
            category: BottleneckCategory::DnsFailure,
            severity: AlertLevel::Critical,
            title: "DNS resolver failures".to_string(),
            message: format!("{failures} configured resolver(s) failed lookup probes."),
            suggestions: vec![
                "Review DNS Assist recommendations.".to_string(),
                "On corporate networks, avoid changing DNS without IT approval.".to_string(),
            ],
        });
    }

    if let Some(slowest) = report
        .probe
        .dns
        .iter()
        .filter(|sample| sample.success)
        .max_by(|left, right| {
            left.latency_ms
                .partial_cmp(&right.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        if slowest.latency_ms >= DNS_SLOW_MS {
            hints.push(BottleneckHint {
                category: BottleneckCategory::DnsSlow,
                severity: AlertLevel::Warning,
                title: "Slow DNS responses".to_string(),
                message: format!(
                    "DNS via {} took {:.0} ms.",
                    slowest.resolver, slowest.latency_ms
                ),
                suggestions: vec![
                    "DNS Assist may reduce lookup latency.".to_string(),
                    "Slow DNS rarely caps download Mbps but can delay page starts.".to_string(),
                ],
            });
        }
    }
}

fn push_integrity_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let Some(integrity) = &report.dns_integrity else {
        return;
    };

    if matches!(
        integrity.state,
        DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
    ) {
        let severity = if matches!(integrity.state, DnsIntegrityState::Suspicious) {
            AlertLevel::Critical
        } else {
            AlertLevel::Warning
        };

        hints.push(BottleneckHint {
            category: BottleneckCategory::DnsIntegrity,
            severity,
            title: "DNS integrity concern".to_string(),
            message: integrity.summary.clone(),
            suggestions: vec![
                "Review mismatched domains on the DNS page.".to_string(),
                "Consider DNS Assist only after understanding corporate split-DNS.".to_string(),
            ],
        });
    }
}

fn push_reachability_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let Some(reachability) = &report.site_reachability else {
        return;
    };

    if !crate::reachability::site_access_degraded(reachability) {
        return;
    }

    let severity = if reachability.success_count == 0 {
        AlertLevel::Critical
    } else {
        AlertLevel::Warning
    };

    let mut suggestions = vec![
        "If you use a proxy, try another node in Connect Assist.".to_string(),
        "Check DNS integrity results for poisoning or split-DNS issues.".to_string(),
    ];

    if report.environment.proxy.enabled {
        suggestions.push("Temporarily disable the proxy to see if sites load on the native path.".to_string());
    }

    hints.push(BottleneckHint {
        category: BottleneckCategory::SiteAccess,
        severity,
        title: "Sites blocked or failing on this path".to_string(),
        message: reachability.summary.clone(),
        suggestions,
    });
}

fn push_context_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let Some(context) = &report.network_context else {
        return;
    };

    if matches!(context.kind, NetworkContextKind::CaptivePortal) {
        hints.push(BottleneckHint {
            category: BottleneckCategory::CaptivePortal,
            severity: AlertLevel::Critical,
            title: "Captive portal detected".to_string(),
            message: context.captive_portal.summary.clone(),
            suggestions: vec![
                "Open a browser and complete the Wi-Fi login page.".to_string(),
                "Re-run a health check after sign-in.".to_string(),
            ],
        });
        return;
    }

    if matches!(
        context.kind,
        NetworkContextKind::GuestWifi | NetworkContextKind::PublicCellular
    ) && matches!(context.risk_level, NetworkRiskLevel::Moderate | NetworkRiskLevel::High)
    {
        let severity = if matches!(context.risk_level, NetworkRiskLevel::High) {
            AlertLevel::Critical
        } else {
            AlertLevel::Warning
        };

        hints.push(BottleneckHint {
            category: BottleneckCategory::PublicNetwork,
            severity,
            title: "Public or guest network".to_string(),
            message: context.summary.clone(),
            suggestions: vec![
                "Use a VPN for sensitive accounts on guest Wi-Fi.".to_string(),
                "Let KnotTrace apply trusted DNS if hijacking is detected.".to_string(),
            ],
        });
    }
}

fn push_egress_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let Some(egress) = &report.egress else {
        return;
    };

    if crate::egress::egress_unstable(egress) {
        hints.push(BottleneckHint {
            category: BottleneckCategory::EgressUnstable,
            severity: AlertLevel::Warning,
            title: "Unstable public IP detection".to_string(),
            message: egress.summary.clone(),
            suggestions: vec![
                "Proxy or VPN paths may be masking egress.".to_string(),
                "Compare results after captive portal sign-in.".to_string(),
            ],
        });
    }
}

fn push_stability_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    if let Some(stability) = &report.stability {
        if let Some(bufferbloat) = &stability.bufferbloat {
            if matches!(
                bufferbloat.grade,
                BufferbloatGrade::Moderate | BufferbloatGrade::Severe
            ) {
                let severity = if matches!(bufferbloat.grade, BufferbloatGrade::Severe) {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                };
                hints.push(BottleneckHint {
                    category: BottleneckCategory::Bufferbloat,
                    severity,
                    title: "Latency spikes under load".to_string(),
                    message: bufferbloat.summary.clone(),
                    suggestions: vec![
                        "Enable router SQM/CAKE and shape to 85–95% of your plan speed.".to_string(),
                        "Prefer Ethernet for video calls and large transfers.".to_string(),
                    ],
                });
            } else if matches!(bufferbloat.grade, BufferbloatGrade::Mild) {
                hints.push(BottleneckHint {
                    category: BottleneckCategory::Bufferbloat,
                    severity: AlertLevel::Info,
                    title: "Mild latency rise under load".to_string(),
                    message: bufferbloat.summary.clone(),
                    suggestions: vec![
                        "Monitor during uploads; bufferbloat may worsen with VPN/Tor.".to_string(),
                    ],
                });
            }
        }

        if let Some(mtu) = &stability.mtu {
            if mtu.fragmentation_risk {
                hints.push(BottleneckHint {
                    category: BottleneckCategory::MtuFragmentation,
                    severity: AlertLevel::Warning,
                    title: "Low path MTU detected".to_string(),
                    message: mtu.summary.clone(),
                    suggestions: vec![
                        "For VPN/Tor apps, try tunnel MTU around 1400 and MSS near 1360.".to_string(),
                        "Large file stalls with normal web browsing often point to MTU issues.".to_string(),
                    ],
                });
            }
        }
    }
}

fn push_path_hints(report: &HealthReport, hints: &mut Vec<BottleneckHint>) {
    let env = &report.environment;
    let on_vpn = env.tags.contains(&EnvironmentTag::Vpn);
    let on_tor = env.tor.detected || env.tags.contains(&EnvironmentTag::Tor);
    let on_proxy = env.proxy.enabled || env.tags.contains(&EnvironmentTag::Proxy);
    let degraded = matches!(report.score.grade, HealthGrade::Fair | HealthGrade::Poor);

    if on_tor {
        let severity = if !env.tor.socks_reachable {
            AlertLevel::Critical
        } else if degraded {
            AlertLevel::Warning
        } else {
            AlertLevel::Info
        };

        hints.push(BottleneckHint {
            category: BottleneckCategory::TorTunnel,
            severity,
            title: "Tor routing active".to_string(),
            message: if let Some(endpoint) = &env.tor.socks_endpoint {
                format!(
                    "Tor SOCKS proxy detected at {endpoint}. Expect higher latency and lower throughput than direct paths."
                )
            } else {
                "Tor environment detected. Expect higher latency and lower throughput than direct paths."
                    .to_string()
            },
            suggestions: vec![
                "Wait for Tor bootstrap to finish before judging app performance.".to_string(),
                "Compare direct vs Tor paths with benchmark snapshots or an on-demand throughput test."
                    .to_string(),
                "If large transfers stall, lower tunnel MTU and verify SOCKS reachability.".to_string(),
            ],
        });
    }

    if on_proxy && !on_tor {
        if let Some(path_report) = &report.proxy_path_report {
            if path_report.proxy_failure_count > 0 || path_report.likely_provider_side {
                let severity = if path_report.likely_provider_side {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                };
                hints.push(BottleneckHint {
                    category: BottleneckCategory::ProxyPath,
                    severity,
                    title: if path_report.likely_provider_side {
                        "Proxy provider path is impairing access".to_string()
                    } else {
                        "Proxy path is breaking major services".to_string()
                    },
                    message: path_report.summary.clone(),
                    suggestions: vec![
                        "Review the Proxy path report on the Network page for per-domain errors."
                            .to_string(),
                        "Switch proxy node or provider manually — KnotTrace cannot repair upstream filtering."
                            .to_string(),
                        "Open Connect Assist to compare nodes if Mihomo/sing-box is configured."
                            .to_string(),
                    ],
                });
            }
        } else if degraded {
            let proxy_major_failures = report
                .site_reachability
                .as_ref()
                .map(crate::reachability::proxy_verification_failures)
                .unwrap_or_default();

            let (severity, title, message) = if !proxy_major_failures.is_empty() {
                let sample = proxy_major_failures
                    .iter()
                    .take(2)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    AlertLevel::Critical,
                    "Proxy path is breaking major services".to_string(),
                    format!(
                        "High-signal sites failed while a proxy is active (example: {sample}). This strongly suggests the proxy node is unstable, filtered, or resetting connections."
                    ),
                )
            } else {
                (
                    AlertLevel::Warning,
                    "Proxy path may limit performance".to_string(),
                    "A system proxy is active and connection quality is not ideal.".to_string(),
                )
            };

            let mut suggestions = vec![
                "Try another node in Connect Assist.".to_string(),
                "Compare direct vs proxy path with a manual check.".to_string(),
            ];
            if !proxy_major_failures.is_empty() {
                suggestions.push(
                    "If this is a corporate proxy/VPN, check for filtering rules or required allowlists."
                        .to_string(),
                );
            }

            hints.push(BottleneckHint {
                category: BottleneckCategory::ProxyPath,
                severity,
                title,
                message,
                suggestions,
            });
        }
    }

    if on_vpn && (degraded || report.probe.internet.as_ref().is_some_and(|sample| sample.avg_ms >= 80.0)) {
        hints.push(BottleneckHint {
            category: BottleneckCategory::VpnTunnel,
            severity: AlertLevel::Warning,
            title: "VPN tunnel overhead".to_string(),
            message: "Traffic appears to traverse a VPN interface with elevated latency or loss.".to_string(),
            suggestions: vec![
                "Large file stalls may be MTU-related — try a lower tunnel MTU.".to_string(),
                "Ask IT about split tunneling if public traffic is slow.".to_string(),
            ],
        });
    }

    let active_kind = active_interface_kind(env);
    if matches!(active_kind, Some(LinkKind::WiFi)) {
        hints.push(BottleneckHint {
            category: BottleneckCategory::WifiPath,
            severity: AlertLevel::Info,
            title: "Active path uses Wi-Fi".to_string(),
            message: "Wi-Fi often caps real-world throughput below ISP plan speeds.".to_string(),
            suggestions: vec![
                "Test on Ethernet to see if Mbps improves.".to_string(),
                "Check router bufferbloat with SQM/CAKE if latency spikes under load.".to_string(),
            ],
        });
    }

    if matches!(active_kind, Some(LinkKind::Cellular)) {
        hints.push(BottleneckHint {
            category: BottleneckCategory::CellularPath,
            severity: AlertLevel::Info,
            title: "Active path uses cellular".to_string(),
            message: "Mobile links vary widely and may not reach wired plan speeds.".to_string(),
            suggestions: vec!["Expect more jitter; VPN may add further overhead.".to_string()],
        });
    }
}

fn active_interface_kind(environment: &EnvironmentSnapshot) -> Option<LinkKind> {
    let active = environment.active_interface.as_deref()?;
    environment
        .interfaces
        .iter()
        .find(|iface| iface.name == active)
        .map(|iface| iface.kind)
}

fn severity_rank(level: AlertLevel) -> u8 {
    match level {
        AlertLevel::Critical => 3,
        AlertLevel::Warning => 2,
        AlertLevel::Info => 1,
    }
}

fn classify_slowdown_shape(report: &HealthReport) -> SlowdownShape {
    let captive = report.network_context.as_ref().is_some_and(|context| {
        matches!(
            context.captive_portal.state,
            CaptivePortalState::Suspected | CaptivePortalState::Confirmed
        ) || matches!(context.kind, NetworkContextKind::CaptivePortal)
    });
    if captive {
        return SlowdownShape::RestrictedNetwork;
    }

    let public_restricted = report.network_context.as_ref().is_some_and(|context| {
        matches!(
            context.kind,
            NetworkContextKind::GuestWifi | NetworkContextKind::PublicCellular
        ) && matches!(context.risk_level, NetworkRiskLevel::High)
    });
    let dns_cluster = dns_symptoms_cluster(report);
    if public_restricted && (dns_cluster || reachability_degraded(report)) {
        return SlowdownShape::RestrictedNetwork;
    }

    if dns_cluster {
        return SlowdownShape::PageStart;
    }

    if severe_bufferbloat(report) {
        return SlowdownShape::UnderLoadLag;
    }

    if reachability_degraded(report) {
        return SlowdownShape::PartialSiteFailure;
    }

    if gateway_issue(report) {
        return SlowdownShape::LinkLocalIssue;
    }

    if tunnel_overhead_suspected(report) {
        return SlowdownShape::TunnelOverhead;
    }

    SlowdownShape::GeneralDegradation
}

fn diagnosis_confidence(report: &HealthReport, shape: SlowdownShape) -> DiagnosisConfidence {
    match shape {
        SlowdownShape::RestrictedNetwork => {
            if report.network_context.as_ref().is_some_and(|context| {
                matches!(context.captive_portal.state, CaptivePortalState::Confirmed)
            }) {
                DiagnosisConfidence::High
            } else if report.network_context.as_ref().is_some_and(|context| {
                matches!(context.risk_level, NetworkRiskLevel::High)
            }) {
                DiagnosisConfidence::Medium
            } else {
                DiagnosisConfidence::Low
            }
        }
        SlowdownShape::PageStart => {
            if report.dns_integrity.as_ref().is_some_and(|integrity| {
                matches!(
                    integrity.state,
                    DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
                )
            }) || report.probe.dns.iter().any(|sample| !sample.success)
            {
                DiagnosisConfidence::High
            } else {
                DiagnosisConfidence::Medium
            }
        }
        SlowdownShape::UnderLoadLag => DiagnosisConfidence::High,
        SlowdownShape::PartialSiteFailure => {
            if report
                .proxy_path_report
                .as_ref()
                .is_some_and(|path| path.likely_provider_side)
            {
                DiagnosisConfidence::High
            } else if report.stability.as_ref().is_some_and(|stability| {
                stability
                    .mtu
                    .as_ref()
                    .is_some_and(|mtu| mtu.fragmentation_risk)
            }) || report.environment.proxy.enabled
                || report.environment.tags.contains(&EnvironmentTag::Vpn)
                || report.environment.tags.contains(&EnvironmentTag::Tor)
            {
                DiagnosisConfidence::High
            } else {
                DiagnosisConfidence::Medium
            }
        }
        SlowdownShape::LinkLocalIssue => DiagnosisConfidence::High,
        SlowdownShape::TunnelOverhead => {
            if report.environment.tor.detected || report.environment.tags.contains(&EnvironmentTag::Vpn) {
                DiagnosisConfidence::Medium
            } else {
                DiagnosisConfidence::Low
            }
        }
        SlowdownShape::GeneralDegradation => DiagnosisConfidence::Low,
    }
}

fn hint_priority(report: &HealthReport, shape: SlowdownShape, hint: &BottleneckHint) -> u8 {
    let base = severity_rank(hint.severity) * 10;
    let bonus = match shape {
        SlowdownShape::RestrictedNetwork => match hint.category {
            BottleneckCategory::CaptivePortal => 80,
            BottleneckCategory::PublicNetwork => 70,
            BottleneckCategory::DnsIntegrity | BottleneckCategory::SiteAccess => 40,
            _ => 0,
        },
        SlowdownShape::PageStart => match hint.category {
            BottleneckCategory::DnsFailure => 80,
            BottleneckCategory::DnsIntegrity => 70,
            BottleneckCategory::DnsSlow => 60,
            BottleneckCategory::CaptivePortal => 55,
            BottleneckCategory::SiteAccess => 35,
            _ => 0,
        },
        SlowdownShape::UnderLoadLag => match hint.category {
            BottleneckCategory::Bufferbloat => 80,
            BottleneckCategory::Gateway => 30,
            BottleneckCategory::WifiPath | BottleneckCategory::VpnTunnel => 10,
            _ => 0,
        },
        SlowdownShape::PartialSiteFailure => match hint.category {
            BottleneckCategory::MtuFragmentation if tunnel_present(report) => 85,
            BottleneckCategory::ProxyPath => 80,
            BottleneckCategory::TorTunnel => 75,
            BottleneckCategory::VpnTunnel => 70,
            BottleneckCategory::SiteAccess => 65,
            BottleneckCategory::DnsIntegrity => 40,
            _ => 0,
        },
        SlowdownShape::TunnelOverhead => match hint.category {
            BottleneckCategory::TorTunnel => 80,
            BottleneckCategory::VpnTunnel => 75,
            BottleneckCategory::ProxyPath => 65,
            BottleneckCategory::MtuFragmentation => 45,
            _ => 0,
        },
        SlowdownShape::LinkLocalIssue => match hint.category {
            BottleneckCategory::Gateway => 80,
            BottleneckCategory::WifiPath => 50,
            BottleneckCategory::InternetLoss => 30,
            _ => 0,
        },
        SlowdownShape::GeneralDegradation => match hint.category {
            BottleneckCategory::InternetUnreachable => 90,
            BottleneckCategory::InternetLoss => 70,
            BottleneckCategory::InternetLatency => 60,
            _ => 0,
        },
    };
    base + bonus
}

fn build_summary(
    primary: Option<BottleneckCategory>,
    hint_count: usize,
    shape: SlowdownShape,
    confidence: DiagnosisConfidence,
) -> String {
    let confidence_note = match confidence {
        DiagnosisConfidence::High => "Strong signals point to this cause.",
        DiagnosisConfidence::Medium => "Current probes support this diagnosis.",
        DiagnosisConfidence::Low => "Evidence is limited, so treat this as guidance.",
    };

    match shape {
        SlowdownShape::RestrictedNetwork => {
            format!("This looks like a restricted or guest network. Complete sign-in or verify the path first. {confidence_note}")
        }
        SlowdownShape::PageStart => {
            format!("Pages are likely slowing down before they start loading. DNS or captive portal behavior is the strongest signal. {confidence_note}")
        }
        SlowdownShape::UnderLoadLag => {
            format!("Your connection has acceptable reachability, but latency spikes badly under load. This looks more like bufferbloat than a bandwidth cap. {confidence_note}")
        }
        SlowdownShape::PartialSiteFailure => {
            format!("Some sites are failing only on the current path. A proxy, tunnel, or MTU issue is more likely than a total outage. {confidence_note}")
        }
        SlowdownShape::TunnelOverhead => {
            format!("A privacy or proxy tunnel appears to be adding overhead on this path. {confidence_note}")
        }
        SlowdownShape::LinkLocalIssue => {
            format!("The strongest signal is close to your device or router, not the wider internet. {confidence_note}")
        }
        SlowdownShape::GeneralDegradation => match primary {
            Some(BottleneckCategory::Healthy) => {
                "No obvious bottlenecks detected from current probes.".to_string()
            }
            Some(category) => format!(
                "Likely bottleneck: {} ({} hint(s) total). {}",
                category_label(category),
                hint_count,
                confidence_note
            ),
            None => format!("{hint_count} network hint(s) detected. {confidence_note}"),
        },
    }
}

fn dns_symptoms_cluster(report: &HealthReport) -> bool {
    let dns_failures = report.probe.dns.iter().any(|sample| !sample.success);
    let dns_slow = report
        .probe
        .dns
        .iter()
        .filter(|sample| sample.success)
        .any(|sample| sample.latency_ms >= DNS_SLOW_MS);
    let integrity_bad = report.dns_integrity.as_ref().is_some_and(|integrity| {
        matches!(
            integrity.state,
            DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
        )
    });
    let captive = report.network_context.as_ref().is_some_and(|context| {
        matches!(
            context.captive_portal.state,
            CaptivePortalState::Suspected | CaptivePortalState::Confirmed
        )
    });

    (dns_slow && (integrity_bad || dns_failures || captive)) || (dns_failures && integrity_bad)
}

fn severe_bufferbloat(report: &HealthReport) -> bool {
    report.stability.as_ref().is_some_and(|stability| {
        stability.bufferbloat.as_ref().is_some_and(|bufferbloat| {
            matches!(
                bufferbloat.grade,
                BufferbloatGrade::Moderate | BufferbloatGrade::Severe
            )
        })
    })
}

fn reachability_degraded(report: &HealthReport) -> bool {
    report
        .site_reachability
        .as_ref()
        .is_some_and(crate::reachability::site_access_degraded)
}

fn gateway_issue(report: &HealthReport) -> bool {
    report.probe.gateway.as_ref().is_some_and(|gateway| {
        gateway.loss_pct >= 50.0 || gateway.avg_ms >= GATEWAY_HIGH_LATENCY_MS
    })
}

fn tunnel_present(report: &HealthReport) -> bool {
    report.environment.proxy.enabled
        || report.environment.tags.contains(&EnvironmentTag::Vpn)
        || report.environment.tags.contains(&EnvironmentTag::Tor)
        || report.environment.tor.detected
}

fn tunnel_overhead_suspected(report: &HealthReport) -> bool {
    let degraded = matches!(report.score.grade, HealthGrade::Fair | HealthGrade::Poor);
    let proxy_or_tunnel = tunnel_present(report);
    proxy_or_tunnel && degraded && !reachability_degraded(report) && !dns_symptoms_cluster(report)
}

fn category_label(category: BottleneckCategory) -> &'static str {
    match category {
        BottleneckCategory::InternetUnreachable => "internet unreachable",
        BottleneckCategory::InternetLoss => "internet loss",
        BottleneckCategory::InternetLatency => "internet latency",
        BottleneckCategory::Gateway => "local gateway",
        BottleneckCategory::DnsFailure => "DNS failures",
        BottleneckCategory::DnsSlow => "slow DNS",
        BottleneckCategory::DnsIntegrity => "DNS integrity",
        BottleneckCategory::SiteAccess => "site access",
        BottleneckCategory::ProxyPath => "proxy path",
        BottleneckCategory::VpnTunnel => "VPN tunnel",
        BottleneckCategory::TorTunnel => "Tor tunnel",
        BottleneckCategory::Bufferbloat => "bufferbloat",
        BottleneckCategory::MtuFragmentation => "MTU fragmentation",
        BottleneckCategory::WifiPath => "Wi-Fi path",
        BottleneckCategory::CellularPath => "cellular path",
        BottleneckCategory::PublicNetwork => "public network",
        BottleneckCategory::CaptivePortal => "captive portal",
        BottleneckCategory::EgressUnstable => "egress instability",
        BottleneckCategory::Healthy => "healthy",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_report(
        grade: HealthGrade,
        internet: Option<LatencySample>,
        dns: Vec<DnsProbe>,
        tags: Vec<EnvironmentTag>,
    ) -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "host".to_string(),
                interfaces: vec![NetworkInterface {
                    name: "wlan0".to_string(),
                    friendly_name: Some("Wi-Fi".to_string()),
                    kind: LinkKind::WiFi,
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
                tags,
                default_gateway: Some("192.168.1.1".to_string()),
                active_interface: Some("wlan0".to_string()),
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet,
                dns,
                duration_ms: 0,
            },
            score: HealthScore {
                grade,
                score: 70,
                summary: "test".to_string(),
                reasons: Vec::new(),
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
            site_reachability: None,
            egress: None,
            network_context: None,
            recommendations: None,
            proxy_path_report: None,
        }
    }

    #[test]
    fn flags_unreachable_internet() {
        let report = sample_report(HealthGrade::Poor, None, Vec::new(), Vec::new());
        let diagnosis = diagnose_network(&report);
        assert_eq!(
            diagnosis.primary_bottleneck,
            Some(BottleneckCategory::InternetUnreachable)
        );
    }

    #[test]
    fn flags_wifi_path_info() {
        let report = sample_report(
            HealthGrade::Good,
            Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 30.0,
                loss_pct: 0.0,
            }),
            vec![DnsProbe {
                resolver: "system".to_string(),
                query: "example.com".to_string(),
                latency_ms: 20.0,
                success: true,
            }],
            Vec::new(),
        );
        let diagnosis = diagnose_network(&report);
        assert!(
            diagnosis
                .hints
                .iter()
                .any(|hint| hint.category == BottleneckCategory::WifiPath)
        );
    }

    #[test]
    fn healthy_when_probes_are_clean_on_ethernet() {
        let mut report = sample_report(
            HealthGrade::Good,
            Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 30.0,
                loss_pct: 0.0,
            }),
            vec![DnsProbe {
                resolver: "system".to_string(),
                query: "example.com".to_string(),
                latency_ms: 20.0,
                success: true,
            }],
            Vec::new(),
        );
        report.environment.interfaces[0].kind = LinkKind::Ethernet;
        report.environment.interfaces[0].name = "eth0".to_string();
        report.environment.active_interface = Some("eth0".to_string());

        let diagnosis = diagnose_network(&report);
        assert_eq!(
            diagnosis.primary_bottleneck,
            Some(BottleneckCategory::Healthy)
        );
    }

    #[test]
    fn prioritizes_dns_for_page_start_slowdowns() {
        let mut report = sample_report(
            HealthGrade::Fair,
            Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 35.0,
                loss_pct: 0.0,
            }),
            vec![DnsProbe {
                resolver: "system".to_string(),
                query: "example.com".to_string(),
                latency_ms: 120.0,
                success: true,
            }],
            Vec::new(),
        );
        report.dns_integrity = Some(DnsIntegrityStatus {
            state: DnsIntegrityState::Caution,
            confidence: DnsIntegrityConfidence::Medium,
            mismatch_count: 1,
            checked_domains: 3,
            summary: "Resolver answers differ from trusted DNS.".to_string(),
            details: Vec::new(),
        });

        let diagnosis = diagnose_network(&report);
        assert_eq!(diagnosis.primary_bottleneck, Some(BottleneckCategory::DnsIntegrity));
        assert!(diagnosis.summary.contains("Pages are likely slowing down before they start loading"));
    }

    #[test]
    fn prioritizes_bufferbloat_over_wifi_path() {
        let mut report = sample_report(
            HealthGrade::Fair,
            Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 40.0,
                loss_pct: 0.0,
            }),
            vec![DnsProbe {
                resolver: "system".to_string(),
                query: "example.com".to_string(),
                latency_ms: 20.0,
                success: true,
            }],
            Vec::new(),
        );
        report.stability = Some(StabilityProbeResult {
            bufferbloat: Some(BufferbloatProbe {
                idle_latency_ms: 18.0,
                loaded_latency_ms: 180.0,
                latency_delta_ms: 162.0,
                grade: BufferbloatGrade::Severe,
                summary: "Latency rises sharply during load.".to_string(),
            }),
            mtu: None,
            duration_ms: 0,
        });

        let diagnosis = diagnose_network(&report);
        assert_eq!(diagnosis.primary_bottleneck, Some(BottleneckCategory::Bufferbloat));
        assert!(diagnosis.summary.contains("bufferbloat"));
    }

    #[test]
    fn prioritizes_captive_portal_over_other_hints() {
        let mut report = sample_report(
            HealthGrade::Poor,
            Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 80.0,
                loss_pct: 0.0,
            }),
            vec![DnsProbe {
                resolver: "system".to_string(),
                query: "example.com".to_string(),
                latency_ms: 25.0,
                success: true,
            }],
            Vec::new(),
        );
        report.network_context = Some(NetworkContextReport {
            kind: NetworkContextKind::CaptivePortal,
            risk_level: NetworkRiskLevel::High,
            captive_portal: CaptivePortalStatus {
                state: CaptivePortalState::Confirmed,
                probe_url: "http://connectivitycheck.gstatic.com/generate_204".to_string(),
                status_code: Some(200),
                redirected: true,
                summary: "Captive portal login page detected before full internet access.".to_string(),
            },
            signals: vec!["Captive portal login page detected".to_string()],
            summary: "Appears to be captive portal network (high risk).".to_string(),
        });

        let diagnosis = diagnose_network(&report);
        assert_eq!(diagnosis.primary_bottleneck, Some(BottleneckCategory::CaptivePortal));
        assert!(diagnosis.summary.contains("restricted or guest network"));
    }

    #[test]
    fn proxy_major_site_failures_become_primary_bottleneck() {
        let mut report = sample_report(
            HealthGrade::Poor,
            Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 60.0,
                loss_pct: 0.0,
            }),
            vec![DnsProbe {
                resolver: "system".to_string(),
                query: "example.com".to_string(),
                latency_ms: 20.0,
                success: true,
            }],
            vec![EnvironmentTag::Proxy],
        );
        report.environment.proxy.enabled = true;
        report.environment.proxy.server = Some("127.0.0.1:7890".to_string());
        report.site_reachability = Some(SiteReachabilityStatus {
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
                    latency_ms: Some(40.0),
                    error: None,
                    error_kind: None,
                },
            ],
            summary: "failed".to_string(),
        });

        let diagnosis = diagnose_network(&report);
        assert_eq!(diagnosis.slowdown_shape, SlowdownShape::PartialSiteFailure);
        assert_eq!(diagnosis.primary_bottleneck, Some(BottleneckCategory::ProxyPath));
        assert!(diagnosis
            .hints
            .first()
            .is_some_and(|hint| hint.title.to_ascii_lowercase().contains("major services")));
    }
}
