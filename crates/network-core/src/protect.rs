use crate::types::*;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtectError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn default_protect_settings() -> ProtectSettings {
    ProtectSettings {
        enabled: true,
        notify_on_grade_drop: true,
        notify_on_untrusted_network: true,
        notify_on_degraded: true,
        auto_apply_dns: true,
        auto_apply_connect: false,
        auto_apply_on_untrusted_only: true,
        auto_recover_dns_integrity: true,
        auto_recover_site_access: false,
    }
}

pub fn settings_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("protect_settings.json")
}

pub fn load_protect_settings(data_dir: &Path) -> Result<ProtectSettings, ProtectError> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(default_protect_settings());
    }

    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save_protect_settings(
    data_dir: &Path,
    settings: &ProtectSettings,
) -> Result<(), ProtectError> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(settings_path(data_dir), serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

pub fn evaluate_protect(
    report: &HealthReport,
    previous_grade: Option<HealthGrade>,
    settings: &ProtectSettings,
) -> ProtectStatus {
    let trust_level = classify_trust_level(&report.environment, &report.score, report.network_context.as_ref());
    let mut alerts = Vec::new();

    if settings.enabled {
        if matches!(trust_level, TrustLevel::Untrusted) {
            alerts.push(untrusted_network_alert(&report.environment, settings));
        }

        if matches!(report.score.grade, HealthGrade::Poor) {
            alerts.push(degraded_connection_alert(&report.score, settings));
        } else if matches!(report.score.grade, HealthGrade::Fair) {
            alerts.push(fair_connection_alert(&report.score, settings));
        }

        if report.environment.proxy.enabled && matches!(report.score.grade, HealthGrade::Fair | HealthGrade::Poor) {
            alerts.push(proxy_path_alert(settings));
        }

        if let Some(previous) = previous_grade {
            if grade_dropped(previous, report.score.grade) {
                alerts.push(grade_drop_alert(previous, report.score.grade, settings));
            }
        }

        if let Some(integrity) = &report.dns_integrity {
            if matches!(
                integrity.state,
                DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
            ) {
                alerts.push(dns_integrity_alert(integrity, settings));
            }
        }

        if let Some(reachability) = &report.site_reachability {
            if crate::reachability::site_access_degraded(reachability) {
                alerts.push(site_access_alert(reachability, settings, &report.environment));
            }
        }

        if let Some(context) = &report.network_context {
            if matches!(context.kind, NetworkContextKind::GuestWifi | NetworkContextKind::PublicCellular)
                && !report.environment.tags.contains(&EnvironmentTag::Vpn)
            {
                alerts.push(guest_network_alert(context, settings));
            }

            if matches!(
                context.captive_portal.state,
                CaptivePortalState::Suspected | CaptivePortalState::Confirmed
            ) {
                alerts.push(captive_portal_alert(context, settings));
            }
        }
    }

    let summary = summarize_status(trust_level, &report.score, alerts.len());

    ProtectStatus {
        trust_level,
        alerts,
        settings: settings.clone(),
        summary,
    }
}

pub fn should_notify(
    status: &ProtectStatus,
    previous_grade: Option<HealthGrade>,
    current_grade: HealthGrade,
    settings: &ProtectSettings,
    dns_integrity: Option<&DnsIntegrityStatus>,
) -> Option<(String, String)> {
    if !settings.enabled {
        return None;
    }

    if let Some(integrity) = dns_integrity {
        if matches!(
            integrity.confidence,
            DnsIntegrityConfidence::Medium | DnsIntegrityConfidence::High
        ) && matches!(
            integrity.state,
            DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
        ) {
            return Some((
                "DNS integrity risk detected".to_string(),
                integrity.summary.clone(),
            ));
        }
    }

    if settings.notify_on_untrusted_network && matches!(status.trust_level, TrustLevel::Untrusted) {
        return Some((
            "Untrusted network detected".to_string(),
            status.summary.clone(),
        ));
    }

    if settings.notify_on_degraded && matches!(current_grade, HealthGrade::Poor) {
        return Some((
            "Connection quality is poor".to_string(),
            status.summary.clone(),
        ));
    }

    if settings.notify_on_grade_drop {
        if let Some(previous) = previous_grade {
            if grade_dropped(previous, current_grade) {
                return Some((
                    "Connection quality dropped".to_string(),
                    format!(
                        "Health changed from {:?} to {:?}.",
                        previous, current_grade
                    ),
                ));
            }
        }
    }

    None
}

fn classify_trust_level(
    environment: &EnvironmentSnapshot,
    score: &HealthScore,
    network_context: Option<&NetworkContextReport>,
) -> TrustLevel {
    let behind_vpn = environment.tags.contains(&EnvironmentTag::Vpn);

    if let Some(context) = network_context {
        if crate::network_context::is_untrusted_context(context, behind_vpn) {
            return TrustLevel::Untrusted;
        }
    }

    let on_public = environment.tags.contains(&EnvironmentTag::Public);
    let on_home = environment.tags.contains(&EnvironmentTag::HomeLan);

    if on_public && !behind_vpn {
        return TrustLevel::Untrusted;
    }

    if !on_home && !behind_vpn && environment.proxy.enabled {
        return TrustLevel::Caution;
    }

    if matches!(score.grade, HealthGrade::Poor) {
        return TrustLevel::Untrusted;
    }

    if matches!(score.grade, HealthGrade::Fair) || environment.proxy.enabled {
        return TrustLevel::Caution;
    }

    TrustLevel::Trusted
}

fn auto_protect_note(settings: &ProtectSettings) -> &'static str {
    if settings.auto_apply_dns || settings.auto_apply_connect {
        " KnotTrace will try to improve this automatically when it is safe to do so."
    } else {
        ""
    }
}

fn untrusted_network_alert(environment: &EnvironmentSnapshot, settings: &ProtectSettings) -> ProtectAlert {
    let on_public = environment.tags.contains(&EnvironmentTag::Public);
    let auto = auto_protect_note(settings);
    let message = if on_public {
        format!(
            "You appear to be on a public or cellular network.{auto} We will keep monitoring and notify you if action is needed."
        )
    } else {
        format!(
            "This network path does not look like a trusted home connection.{auto}"
        )
    };

    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Untrusted network".to_string(),
        message,
        actions: alert_actions(settings),
    }
}

fn degraded_connection_alert(score: &HealthScore, settings: &ProtectSettings) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Critical,
        title: "Connection degraded".to_string(),
        message: format!(
            "{} {}{}",
            score.summary,
            score.reasons.join("; "),
            auto_protect_note(settings)
        ),
        actions: alert_actions(settings),
    }
}

fn fair_connection_alert(score: &HealthScore, settings: &ProtectSettings) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Info,
        title: "Connection could be smoother".to_string(),
        message: format!("{}{}", score.summary, auto_protect_note(settings)),
        actions: alert_actions(settings),
    }
}

fn proxy_path_alert(settings: &ProtectSettings) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Proxy path needs attention".to_string(),
        message: format!(
            "A system proxy is active and connection quality is not ideal.{}",
            auto_protect_note(settings)
        ),
        actions: alert_actions(settings),
    }
}

fn grade_drop_alert(previous: HealthGrade, current: HealthGrade, settings: &ProtectSettings) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Health score dropped".to_string(),
        message: format!(
            "Connection health changed from {previous:?} to {current:?}.{}",
            auto_protect_note(settings)
        ),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::RunCheck,
            label: "View latest results".to_string(),
        }],
    }
}

fn dns_integrity_alert(integrity: &DnsIntegrityStatus, settings: &ProtectSettings) -> ProtectAlert {
    let level = match integrity.state {
        DnsIntegrityState::Suspicious => AlertLevel::Critical,
        DnsIntegrityState::Caution => AlertLevel::Warning,
        DnsIntegrityState::Ok => AlertLevel::Info,
    };

    ProtectAlert {
        level,
        title: "DNS integrity concern".to_string(),
        message: format!(
            "{} Smart protect can apply trusted DNS automatically when integrity recovery is enabled.{}",
            integrity.summary,
            if settings.auto_recover_dns_integrity && settings.auto_apply_dns {
                " Recovery may run even on familiar networks."
            } else {
                ""
            }
        ),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::RunCheck,
            label: "View details".to_string(),
        }],
    }
}

fn site_access_alert(
    reachability: &SiteReachabilityStatus,
    settings: &ProtectSettings,
    environment: &EnvironmentSnapshot,
) -> ProtectAlert {
    let level = if reachability.success_count == 0 {
        AlertLevel::Critical
    } else {
        AlertLevel::Warning
    };

    let proxy_note = if environment.proxy.enabled {
        " Proxy is active — review Connect Assist if sites fail."
    } else {
        ""
    };

    let auto_note = if settings.auto_recover_site_access && settings.auto_apply_connect {
        " Site-access recovery is enabled."
    } else {
        ""
    };

    ProtectAlert {
        level,
        title: "Sites unreachable on current path".to_string(),
        message: format!("{}{}{}", reachability.summary, proxy_note, auto_note),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::RunCheck,
            label: "View details".to_string(),
        }],
    }
}

fn guest_network_alert(context: &NetworkContextReport, settings: &ProtectSettings) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Guest or public network".to_string(),
        message: format!(
            "{} Avoid sensitive logins unless a VPN is active.{}",
            context.summary,
            auto_protect_note(settings)
        ),
        actions: alert_actions(settings),
    }
}

fn captive_portal_alert(context: &NetworkContextReport, settings: &ProtectSettings) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Critical,
        title: "Wi-Fi sign-in required".to_string(),
        message: format!(
            "{} Complete the hotspot login in your browser before trusting this path.{}",
            context.captive_portal.summary,
            auto_protect_note(settings)
        ),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::RunCheck,
            label: "Re-check after login".to_string(),
        }],
    }
}

fn alert_actions(settings: &ProtectSettings) -> Vec<ProtectAction> {
    if settings.auto_apply_dns || settings.auto_apply_connect {
        return Vec::new();
    }

    vec![
        ProtectAction {
            kind: ProtectActionKind::DnsAssist,
            label: "Try DNS Assist".to_string(),
        },
        ProtectAction {
            kind: ProtectActionKind::ConnectAssist,
            label: "Try Connect Assist".to_string(),
        },
    ]
}

fn grade_dropped(previous: HealthGrade, current: HealthGrade) -> bool {
    matches!(
        (previous, current),
        (HealthGrade::Good, HealthGrade::Fair)
            | (HealthGrade::Good, HealthGrade::Poor)
            | (HealthGrade::Fair, HealthGrade::Poor)
    )
}

fn summarize_status(trust_level: TrustLevel, score: &HealthScore, alert_count: usize) -> String {
    let trust = match trust_level {
        TrustLevel::Trusted => "trusted",
        TrustLevel::Caution => "use caution on",
        TrustLevel::Untrusted => "untrusted",
    };

    if alert_count == 0 {
        format!(
            "Network looks {trust}. Current health is {:?} ({}).",
            score.grade, score.score
        )
    } else {
        format!(
            "Network looks {trust}. {:?} health with {alert_count} active alert(s).",
            score.grade
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_report(tags: Vec<EnvironmentTag>, grade: HealthGrade) -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "host".to_string(),
                interfaces: Vec::new(),
                dns_servers: Vec::new(),
                proxy: ProxySettings {
                    enabled: false,
                    server: None,
                    source: "none".to_string(),
                },
                tags,
                default_gateway: Some("10.0.0.1".to_string()),
                active_interface: None,
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: None,
                dns: Vec::new(),
                duration_ms: 0,
            },
            score: HealthScore {
                grade,
                score: 70,
                summary: "test".to_string(),
                reasons: vec!["example".to_string()],
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
            site_reachability: None,
            egress: None,
            network_context: None,
            recommendations: None,
        }
    }

    #[test]
    fn flags_public_network_as_untrusted() {
        let report = sample_report(vec![EnvironmentTag::Public], HealthGrade::Good);
        let status = evaluate_protect(&report, None, &default_protect_settings());
        assert_eq!(status.trust_level, TrustLevel::Untrusted);
        assert!(!status.alerts.is_empty());
    }

    #[test]
    fn flags_guest_wifi_as_untrusted() {
        let mut report = sample_report(vec![EnvironmentTag::HomeLan], HealthGrade::Good);
        report.network_context = Some(NetworkContextReport {
            kind: NetworkContextKind::GuestWifi,
            risk_level: NetworkRiskLevel::Moderate,
            captive_portal: CaptivePortalStatus {
                state: CaptivePortalState::NotDetected,
                probe_url: "http://example".to_string(),
                status_code: Some(204),
                redirected: false,
                summary: "clear".to_string(),
            },
            signals: vec!["Wi-Fi".to_string()],
            summary: "guest wifi".to_string(),
        });

        let status = evaluate_protect(&report, None, &default_protect_settings());
        assert_eq!(status.trust_level, TrustLevel::Untrusted);
    }

    #[test]
    fn detects_grade_drop() {
        assert!(grade_dropped(HealthGrade::Good, HealthGrade::Fair));
        assert!(!grade_dropped(HealthGrade::Fair, HealthGrade::Fair));
    }

    #[test]
    fn adds_dns_integrity_alert_for_suspicious_state() {
        let mut report = sample_report(vec![EnvironmentTag::HomeLan], HealthGrade::Good);
        report.dns_integrity = Some(DnsIntegrityStatus {
            state: DnsIntegrityState::Suspicious,
            confidence: DnsIntegrityConfidence::High,
            mismatch_count: 2,
            checked_domains: 4,
            summary: "Likely DNS tampering".to_string(),
            details: Vec::new(),
        });

        let status = evaluate_protect(&report, None, &default_protect_settings());
        assert!(
            status
                .alerts
                .iter()
                .any(|alert| alert.title == "DNS integrity concern")
        );
    }
}
