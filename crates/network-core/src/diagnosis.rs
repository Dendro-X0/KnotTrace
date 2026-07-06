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
    push_stability_hints(report, &mut hints);
    push_path_hints(report, &mut hints);

    if hints.is_empty() {
        return NetworkDiagnosis {
            summary: "No obvious bottlenecks detected from current probes.".to_string(),
            primary_bottleneck: Some(BottleneckCategory::Healthy),
            hints: vec![healthy_hint()],
        };
    }

    hints.sort_by(|left, right| severity_rank(right.severity).cmp(&severity_rank(left.severity)));

    let primary_bottleneck = hints.first().map(|hint| hint.category);
    let summary = build_summary(primary_bottleneck, hints.len());

    NetworkDiagnosis {
        summary,
        primary_bottleneck,
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

    if on_proxy && degraded && !on_tor {
        hints.push(BottleneckHint {
            category: BottleneckCategory::ProxyPath,
            severity: AlertLevel::Warning,
            title: "Proxy path may limit throughput".to_string(),
            message: "A system proxy is active and connection quality is not ideal.".to_string(),
            suggestions: vec![
                "Try another node in Connect Assist.".to_string(),
                "Compare Speedtest with proxy on vs off.".to_string(),
            ],
        });
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

fn build_summary(primary: Option<BottleneckCategory>, hint_count: usize) -> String {
    match primary {
        Some(BottleneckCategory::Healthy) => {
            "No obvious bottlenecks detected from current probes.".to_string()
        }
        Some(category) => format!(
            "Likely bottleneck: {} ({} hint(s) total).",
            category_label(category),
            hint_count
        ),
        None => format!("{hint_count} network hint(s) detected."),
    }
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
        BottleneckCategory::ProxyPath => "proxy path",
        BottleneckCategory::VpnTunnel => "VPN tunnel",
        BottleneckCategory::TorTunnel => "Tor tunnel",
        BottleneckCategory::Bufferbloat => "bufferbloat",
        BottleneckCategory::MtuFragmentation => "MTU fragmentation",
        BottleneckCategory::WifiPath => "Wi-Fi path",
        BottleneckCategory::CellularPath => "cellular path",
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
}
