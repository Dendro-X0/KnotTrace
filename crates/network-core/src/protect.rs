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
        auto_apply_dns: false,
        auto_apply_connect: false,
        auto_apply_on_untrusted_only: true,
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
    let trust_level = classify_trust_level(&report.environment, &report.score);
    let mut alerts = Vec::new();

    if settings.enabled {
        if matches!(trust_level, TrustLevel::Untrusted) {
            alerts.push(untrusted_network_alert(&report.environment));
        }

        if matches!(report.score.grade, HealthGrade::Poor) {
            alerts.push(degraded_connection_alert(&report.score));
        } else if matches!(report.score.grade, HealthGrade::Fair) {
            alerts.push(fair_connection_alert(&report.score));
        }

        if report.environment.proxy.enabled && matches!(report.score.grade, HealthGrade::Fair | HealthGrade::Poor) {
            alerts.push(proxy_path_alert());
        }

        if let Some(previous) = previous_grade {
            if grade_dropped(previous, report.score.grade) {
                alerts.push(grade_drop_alert(previous, report.score.grade));
            }
        }

        if let Some(integrity) = &report.dns_integrity {
            if matches!(
                integrity.state,
                DnsIntegrityState::Caution | DnsIntegrityState::Suspicious
            ) {
                alerts.push(dns_integrity_alert(integrity));
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

fn classify_trust_level(environment: &EnvironmentSnapshot, score: &HealthScore) -> TrustLevel {
    let on_public = environment.tags.contains(&EnvironmentTag::Public);
    let on_home = environment.tags.contains(&EnvironmentTag::HomeLan);
    let behind_vpn = environment.tags.contains(&EnvironmentTag::Vpn);

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

fn untrusted_network_alert(environment: &EnvironmentSnapshot) -> ProtectAlert {
    let on_public = environment.tags.contains(&EnvironmentTag::Public);
    let message = if on_public {
        "You appear to be on a public or cellular network without VPN protection. Consider enabling your proxy or VPN before sensitive traffic."
    } else {
        "This network path does not look like a trusted home connection."
    };

    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Untrusted network".to_string(),
        message: message.to_string(),
        actions: vec![
            ProtectAction {
                kind: ProtectActionKind::ConnectAssist,
                label: "Review proxy options".to_string(),
            },
            ProtectAction {
                kind: ProtectActionKind::RunCheck,
                label: "Run another health check".to_string(),
            },
        ],
    }
}

fn degraded_connection_alert(score: &HealthScore) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Critical,
        title: "Connection degraded".to_string(),
        message: format!("{} {}", score.summary, score.reasons.join("; ")),
        actions: vec![
            ProtectAction {
                kind: ProtectActionKind::DnsAssist,
                label: "Try DNS Assist".to_string(),
            },
            ProtectAction {
                kind: ProtectActionKind::ConnectAssist,
                label: "Try Connect Assist".to_string(),
            },
        ],
    }
}

fn fair_connection_alert(score: &HealthScore) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Info,
        title: "Connection could be smoother".to_string(),
        message: score.summary.clone(),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::DnsAssist,
            label: "Check DNS Assist".to_string(),
        }],
    }
}

fn proxy_path_alert() -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Proxy path needs attention".to_string(),
        message:
            "A system proxy is active and connection quality is not ideal. A faster node may help."
                .to_string(),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::ConnectAssist,
            label: "Review proxy nodes".to_string(),
        }],
    }
}

fn grade_drop_alert(previous: HealthGrade, current: HealthGrade) -> ProtectAlert {
    ProtectAlert {
        level: AlertLevel::Warning,
        title: "Health score dropped".to_string(),
        message: format!(
            "Connection health changed from {previous:?} to {current:?}."
        ),
        actions: vec![ProtectAction {
            kind: ProtectActionKind::RunCheck,
            label: "Inspect latest results".to_string(),
        }],
    }
}

fn dns_integrity_alert(integrity: &DnsIntegrityStatus) -> ProtectAlert {
    let level = match integrity.state {
        DnsIntegrityState::Suspicious => AlertLevel::Critical,
        DnsIntegrityState::Caution => AlertLevel::Warning,
        DnsIntegrityState::Ok => AlertLevel::Info,
    };

    ProtectAlert {
        level,
        title: "DNS integrity risk detected".to_string(),
        message: format!(
            "{} Confidence: {:?}. {} of {} checked domains mismatched.",
            integrity.summary,
            integrity.confidence,
            integrity.mismatch_count,
            integrity.checked_domains
        ),
        actions: vec![
            ProtectAction {
                kind: ProtectActionKind::DnsAssist,
                label: "Review DNS Assist".to_string(),
            },
            ProtectAction {
                kind: ProtectActionKind::RunCheck,
                label: "Run another health check".to_string(),
            },
            ProtectAction {
                kind: ProtectActionKind::ConnectAssist,
                label: "Review proxy options".to_string(),
            },
        ],
    }
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
                .any(|alert| alert.title == "DNS integrity risk detected")
        );
    }
}
