use crate::types::*;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct CaptivePortalProbeOptions {
    pub timeout: Duration,
}

impl Default for CaptivePortalProbeOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(4),
        }
    }
}

#[derive(Debug, Clone)]
struct CaptiveProbeTarget {
    url: &'static str,
    expect_no_content: bool,
}

const CAPTIVE_TARGETS: &[CaptiveProbeTarget] = &[
    CaptiveProbeTarget {
        url: "http://connectivitycheck.gstatic.com/generate_204",
        expect_no_content: true,
    },
    CaptiveProbeTarget {
        url: "http://www.msftconnecttest.com/connecttest.txt",
        expect_no_content: false,
    },
];

pub async fn probe_captive_portal_with_options(
    options: CaptivePortalProbeOptions,
) -> CaptivePortalStatus {
    let client = match reqwest::Client::builder()
        .timeout(options.timeout)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return CaptivePortalStatus {
                state: CaptivePortalState::NotDetected,
                probe_url: CAPTIVE_TARGETS[0].url.to_string(),
                status_code: None,
                redirected: false,
                summary: format!("Captive portal probe unavailable: {error}"),
            };
        }
    };

    for target in CAPTIVE_TARGETS {
        let response = client.get(target.url).send().await;
        match response {
            Ok(response) => {
                let status = response.status().as_u16();
                let final_url = response.url().to_string();
                let redirected = final_url != target.url;
                let body = response.text().await.unwrap_or_default();

                let portal_detected = if target.expect_no_content {
                    status != 204 || redirected
                } else {
                    status != 200
                        || redirected
                        || !body.to_ascii_lowercase().contains("microsoft")
                };

                if portal_detected {
                    return CaptivePortalStatus {
                        state: if redirected {
                            CaptivePortalState::Confirmed
                        } else {
                            CaptivePortalState::Suspected
                        },
                        probe_url: target.url.to_string(),
                        status_code: Some(status),
                        redirected,
                        summary: if redirected {
                            "Captive portal login page detected before full internet access."
                                .to_string()
                        } else {
                            "Network may require sign-in before internet access.".to_string()
                        },
                    };
                }

                return CaptivePortalStatus {
                    state: CaptivePortalState::NotDetected,
                    probe_url: target.url.to_string(),
                    status_code: Some(status),
                    redirected: false,
                    summary: "No captive portal detected.".to_string(),
                };
            }
            Err(_) => continue,
        }
    }

    CaptivePortalStatus {
        state: CaptivePortalState::NotDetected,
        probe_url: CAPTIVE_TARGETS[0].url.to_string(),
        status_code: None,
        redirected: false,
        summary: "Captive portal probe inconclusive.".to_string(),
    }
}

pub async fn probe_captive_portal() -> CaptivePortalStatus {
    probe_captive_portal_with_options(CaptivePortalProbeOptions::default()).await
}

pub fn assess_network_context(
    environment: &EnvironmentSnapshot,
    captive: &CaptivePortalStatus,
    dns_integrity: Option<&DnsIntegrityStatus>,
    site_reachability: Option<&SiteReachabilityStatus>,
    probe: &ProbeResult,
) -> NetworkContextReport {
    let active_kind = active_interface_kind(environment);
    let private_gateway = environment
        .default_gateway
        .as_deref()
        .is_some_and(is_private_ipv4);
    let behind_vpn = environment.tags.contains(&EnvironmentTag::Vpn);

    let mut signals = Vec::new();

    if matches!(active_kind, Some(LinkKind::WiFi)) {
        signals.push("Active path uses Wi-Fi".to_string());
    }
    if private_gateway {
        signals.push("Private local gateway".to_string());
    }
    if behind_vpn {
        signals.push("VPN tunnel detected".to_string());
    }
    if matches!(captive.state, CaptivePortalState::Suspected | CaptivePortalState::Confirmed) {
        signals.push(captive.summary.clone());
    }
    if dns_integrity.is_some_and(|integrity| {
        matches!(
            integrity.state,
            DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
        )
    }) {
        signals.push("DNS integrity concern".to_string());
    }
    if site_reachability.is_some_and(crate::reachability::site_access_degraded) {
        signals.push("Verification sites failing".to_string());
    }
    if probe.internet.is_none() {
        signals.push("Internet probe unreachable".to_string());
    }

    let kind = if matches!(
        captive.state,
        CaptivePortalState::Suspected | CaptivePortalState::Confirmed
    ) {
        NetworkContextKind::CaptivePortal
    } else if matches!(active_kind, Some(LinkKind::Cellular)) {
        NetworkContextKind::PublicCellular
    } else if matches!(active_kind, Some(LinkKind::WiFi)) && private_gateway {
        NetworkContextKind::GuestWifi
    } else if matches!(active_kind, Some(LinkKind::Ethernet)) && private_gateway {
        NetworkContextKind::HomeLan
    } else if environment.tags.contains(&EnvironmentTag::Public) {
        NetworkContextKind::PublicCellular
    } else if private_gateway {
        NetworkContextKind::HomeLan
    } else {
        NetworkContextKind::Unknown
    };

    let risk_level = classify_risk(kind, behind_vpn, dns_integrity, site_reachability, probe);
    let summary = summarize_context(kind, risk_level, behind_vpn);

    NetworkContextReport {
        kind,
        risk_level,
        captive_portal: captive.clone(),
        signals,
        summary,
    }
}

fn classify_risk(
    kind: NetworkContextKind,
    behind_vpn: bool,
    dns_integrity: Option<&DnsIntegrityStatus>,
    site_reachability: Option<&SiteReachabilityStatus>,
    probe: &ProbeResult,
) -> NetworkRiskLevel {
    if matches!(kind, NetworkContextKind::CaptivePortal) {
        return NetworkRiskLevel::High;
    }

    let integrity_bad = dns_integrity.is_some_and(|integrity| {
        matches!(
            integrity.state,
            DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
        )
    });
    let sites_bad = site_reachability.is_some_and(crate::reachability::site_access_degraded);
    let internet_bad = probe.internet.is_none();

    if internet_bad || integrity_bad {
        return NetworkRiskLevel::High;
    }

    if matches!(
        kind,
        NetworkContextKind::GuestWifi | NetworkContextKind::PublicCellular
    ) {
        if behind_vpn {
            return if sites_bad {
                NetworkRiskLevel::Moderate
            } else {
                NetworkRiskLevel::Low
            };
        }
        return if sites_bad {
            NetworkRiskLevel::High
        } else {
            NetworkRiskLevel::Moderate
        };
    }

    if sites_bad {
        return NetworkRiskLevel::Moderate;
    }

    NetworkRiskLevel::Low
}

fn summarize_context(
    kind: NetworkContextKind,
    risk: NetworkRiskLevel,
    behind_vpn: bool,
) -> String {
    let kind_label = match kind {
        NetworkContextKind::HomeLan => "home or office LAN",
        NetworkContextKind::GuestWifi => "guest or public Wi-Fi",
        NetworkContextKind::PublicCellular => "cellular data",
        NetworkContextKind::CaptivePortal => "captive portal network",
        NetworkContextKind::Unknown => "unknown network",
    };

    let risk_label = match risk {
        NetworkRiskLevel::Low => "low risk",
        NetworkRiskLevel::Moderate => "moderate risk",
        NetworkRiskLevel::High => "high risk",
    };

    if behind_vpn && matches!(kind, NetworkContextKind::GuestWifi | NetworkContextKind::PublicCellular) {
        format!("{kind_label} with VPN active ({risk_label}).")
    } else {
        format!("Appears to be {kind_label} ({risk_label}).")
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

fn is_private_ipv4(address: &str) -> bool {
    address.starts_with("10.")
        || address.starts_with("192.168.")
        || address.starts_with("127.")
        || address
            .strip_prefix("172.")
            .and_then(|rest| rest.split('.').next())
            .and_then(|octet| octet.parse::<u8>().ok())
            .is_some_and(|second| (16..=31).contains(&second))
}

pub fn is_untrusted_context(context: &NetworkContextReport, behind_vpn: bool) -> bool {
    if behind_vpn && matches!(context.risk_level, NetworkRiskLevel::Low) {
        return false;
    }

    matches!(
        context.kind,
        NetworkContextKind::GuestWifi
            | NetworkContextKind::PublicCellular
            | NetworkContextKind::CaptivePortal
    ) || matches!(context.risk_level, NetworkRiskLevel::High)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_environment(kind: LinkKind, gateway: &str) -> EnvironmentSnapshot {
        EnvironmentSnapshot {
            hostname: "host".to_string(),
            interfaces: vec![NetworkInterface {
                name: "wlan0".to_string(),
                friendly_name: Some("Wi-Fi".to_string()),
                kind,
                is_up: true,
                ipv4: vec!["192.168.1.10".to_string()],
                gateway: Some(gateway.to_string()),
                is_default_route: true,
            }],
            dns_servers: Vec::new(),
            proxy: ProxySettings {
                enabled: false,
                server: None,
                source: "none".to_string(),
            },
            tags: Vec::new(),
            default_gateway: Some(gateway.to_string()),
            active_interface: Some("wlan0".to_string()),
            tor: default_tor_status(),
        }
    }

    #[test]
    fn classifies_guest_wifi_on_private_gateway() {
        let env = sample_environment(LinkKind::WiFi, "192.168.50.1");
        let captive = CaptivePortalStatus {
            state: CaptivePortalState::NotDetected,
            probe_url: "http://example".to_string(),
            status_code: Some(204),
            redirected: false,
            summary: "clear".to_string(),
        };
        let report = assess_network_context(&env, &captive, None, None, &ProbeResult {
            gateway: None,
            internet: Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 30.0,
                loss_pct: 0.0,
            }),
            dns: Vec::new(),
            duration_ms: 0,
        });

        assert_eq!(report.kind, NetworkContextKind::GuestWifi);
        assert_eq!(report.risk_level, NetworkRiskLevel::Moderate);
    }

    #[test]
    fn captive_portal_is_high_risk() {
        let env = sample_environment(LinkKind::WiFi, "10.0.0.1");
        let captive = CaptivePortalStatus {
            state: CaptivePortalState::Confirmed,
            probe_url: "http://login".to_string(),
            status_code: Some(200),
            redirected: true,
            summary: "login".to_string(),
        };
        let report = assess_network_context(&env, &captive, None, None, &ProbeResult {
            gateway: None,
            internet: None,
            dns: Vec::new(),
            duration_ms: 0,
        });

        assert_eq!(report.kind, NetworkContextKind::CaptivePortal);
        assert_eq!(report.risk_level, NetworkRiskLevel::High);
        assert!(is_untrusted_context(&report, false));
    }
}
