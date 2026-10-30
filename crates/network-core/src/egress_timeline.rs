//! Path Truth Band 5 — egress anomaly timeline from local health history.

use crate::egress::egress_unstable;
use crate::types::*;
use std::collections::BTreeSet;

const HISTORY_WINDOW: usize = 48;
const MAX_EVENTS: usize = 12;

pub fn evaluate_egress_timeline_snapshot(current: &HealthReport) -> EgressTimelineReport {
    evaluate_egress_timeline(current, &[])
}

pub fn evaluate_egress_timeline(
    current: &HealthReport,
    history: &[HealthReport],
) -> EgressTimelineReport {
    let current_ip = current
        .egress
        .as_ref()
        .and_then(|egress| egress.primary_ip.clone());

    if current.egress.is_none() {
        return report(
            EgressTimelineClaim::None,
            EgressTimelineConfidence::Low,
            "No egress sample",
            "This check did not include an egress probe result.",
            current_ip,
            None,
            0,
            0,
            Vec::new(),
            Vec::new(),
            vec!["Egress probe on this check.".to_string()],
            "Run a full health check to capture public IP.",
        );
    }

    let mut series: Vec<&HealthReport> = history.iter().collect();
    if !series
        .iter()
        .any(|report| report.timestamp == current.timestamp)
    {
        series.insert(0, current);
    }
    series.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
    series.truncate(HISTORY_WINDOW);

    let events = build_events(&series);
    let ips: Vec<String> = series
        .iter()
        .filter_map(|report| {
            report
                .egress
                .as_ref()
                .and_then(|egress| egress.primary_ip.clone())
        })
        .collect();
    let distinct_ips = ips.iter().cloned().collect::<BTreeSet<_>>().len() as u8;
    let checks_considered = series.len() as u8;
    let previous_ip = ips.get(1).cloned();

    if let Some(egress) = &current.egress {
        if egress_unstable(egress) {
            return report(
                EgressTimelineClaim::Unstable,
                EgressTimelineConfidence::Medium,
                "Egress sample looks unstable",
                "Current public IP probe has low confidence or missing agreement across providers. Treat the address as provisional.",
                current_ip,
                previous_ip,
                distinct_ips,
                checks_considered,
                events,
                vec![egress.summary.clone()],
                vec![
                    "Long-term ISP assignment meaning.".to_string(),
                    "Process ↔ egress correlation.".to_string(),
                ],
                "Re-run a full check; compare Tor/system paths on Network if privacy routing matters.",
            );
        }
    }

    if ips.len() < 2 {
        return report(
            EgressTimelineClaim::InsufficientHistory,
            EgressTimelineConfidence::Low,
            "Need more egress history",
            "Fewer than two public-IP samples are stored yet. Timeline claims stay weak until the monitor or manual checks accumulate history.",
            current_ip,
            previous_ip,
            distinct_ips,
            checks_considered,
            events,
            vec![format!("Only {} egress IP sample(s) in window.", ips.len())],
            vec!["Multi-check IP stability.".to_string()],
            "Keep background monitoring on, or run additional checks over time.",
        );
    }

    if distinct_ips >= 3 {
        return report(
            EgressTimelineClaim::Churning,
            EgressTimelineConfidence::High,
            "Public egress IP is churning",
            format!(
                "{distinct_ips} distinct public IPs appeared across {checks_considered} recent checks. This can be CGNAT, VPN/proxy rotation, or carrier reassignment — not proof of interception."
            ),
            current_ip,
            previous_ip,
            distinct_ips,
            checks_considered,
            events,
            vec![format!("Distinct IPs in window: {distinct_ips}.")],
            vec![
                "Which app caused the change.".to_string(),
                "Covert tap detection.".to_string(),
            ],
            "Review Path break / Integrity and whether a VPN or proxy pool is rotating exits.",
        );
    }

    if let (Some(cur), Some(prev)) = (current_ip.clone(), previous_ip.clone()) {
        if cur != prev {
            return report(
                EgressTimelineClaim::IpChanged,
                EgressTimelineConfidence::High,
                "Public egress IP changed",
                format!("Egress moved from {prev} to {cur} since the previous sample."),
                Some(cur.clone()),
                Some(prev.clone()),
                distinct_ips,
                checks_considered,
                events,
                vec![format!("Previous IP {prev}; current IP {cur}.")],
                vec!["Why the ISP or tunnel changed the address.".to_string()],
                "If unexpected, check VPN/proxy status and Integrity indicators.",
            );
        }
    }

    report(
        EgressTimelineClaim::Stable,
        EgressTimelineConfidence::Medium,
        "Public egress IP looks stable",
        format!(
            "Recent checks agree on {} across {checks_considered} samples.",
            current_ip.as_deref().unwrap_or("unknown")
        ),
        current_ip,
        previous_ip,
        distinct_ips,
        checks_considered,
        events,
        vec![format!("Distinct IPs in window: {distinct_ips}.")],
        vec![
            "Silent interception with unchanged egress IP.".to_string(),
            "Future IP changes after this window.".to_string(),
        ],
        "No egress-timeline action needed.",
    )
}

fn build_events(series: &[&HealthReport]) -> Vec<EgressTimelineEvent> {
    let mut events = Vec::new();
    let mut last_ip: Option<String> = None;

    // series is newest-first; walk oldest→newest for chronological transitions, then reverse.
    let mut chronological: Vec<&&HealthReport> = series.iter().collect();
    chronological.reverse();

    for report in chronological {
        let ip = report
            .egress
            .as_ref()
            .and_then(|egress| egress.primary_ip.clone());
        let unstable = report
            .egress
            .as_ref()
            .map(egress_unstable)
            .unwrap_or(true);

        if unstable {
            events.push(EgressTimelineEvent {
                at: report.timestamp,
                kind: EgressTimelineEventKind::UnstableSample,
                detail: report
                    .egress
                    .as_ref()
                    .map(|egress| egress.summary.clone())
                    .unwrap_or_else(|| "Egress sample missing or unstable.".to_string()),
            });
        }

        if let Some(current) = &ip {
            if let Some(prev) = &last_ip {
                if prev != current {
                    events.push(EgressTimelineEvent {
                        at: report.timestamp,
                        kind: EgressTimelineEventKind::IpChanged,
                        detail: format!("Egress changed {prev} → {current}"),
                    });
                }
            }
            last_ip = Some(current.clone());
        }
    }

    events.reverse();
    events.truncate(MAX_EVENTS);
    events
}

#[allow(clippy::too_many_arguments)]
fn report(
    claim: EgressTimelineClaim,
    confidence: EgressTimelineConfidence,
    title: impl Into<String>,
    summary: impl Into<String>,
    current_ip: Option<String>,
    previous_ip: Option<String>,
    distinct_ips: u8,
    checks_considered: u8,
    events: Vec<EgressTimelineEvent>,
    evidence: Vec<String>,
    not_proven: Vec<String>,
    action: impl Into<String>,
) -> EgressTimelineReport {
    EgressTimelineReport {
        claim,
        confidence,
        title: title.into(),
        summary: summary.into(),
        current_ip,
        previous_ip,
        distinct_ips,
        checks_considered,
        events,
        evidence,
        not_proven,
        action: action.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn bare(ip: Option<&str>, minutes_ago: i64, unstable: bool) -> HealthReport {
        let confidence = if unstable {
            EgressConfidence::Low
        } else if ip.is_some() {
            EgressConfidence::High
        } else {
            EgressConfidence::Unknown
        };
        HealthReport {
            timestamp: Utc::now() - Duration::minutes(minutes_ago),
            environment: EnvironmentSnapshot {
                hostname: "h".into(),
                interfaces: vec![],
                dns_servers: vec![],
                proxy: ProxySettings {
                    enabled: false,
                    server: None,
                    source: "none".into(),
                },
                tags: vec![],
                default_gateway: None,
                active_interface: None,
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: None,
                dns: vec![],
                duration_ms: 0,
            },
            score: HealthScore {
                grade: HealthGrade::Good,
                score: 90,
                summary: "ok".into(),
                reasons: vec![],
            },
            dns_integrity: None,
            diagnosis: None,
            stability: None,
            site_reachability: None,
            egress: Some(EgressReport {
                primary_ip: ip.map(str::to_string),
                confidence,
                system_path: EgressPathReport {
                    kind: EgressPathKind::System,
                    primary_ip: ip.map(str::to_string),
                    endpoints: vec![],
                    summary: "system".into(),
                },
                tor_path: None,
                summary: format!("egress {:?}", ip),
            }),
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
    fn stable_across_history() {
        let current = bare(Some("1.1.1.1"), 0, false);
        let history = vec![
            bare(Some("1.1.1.1"), 10, false),
            bare(Some("1.1.1.1"), 20, false),
        ];
        let report = evaluate_egress_timeline(&current, &history);
        assert_eq!(report.claim, EgressTimelineClaim::Stable);
        assert_eq!(report.distinct_ips, 1);
    }

    #[test]
    fn detects_ip_change() {
        let current = bare(Some("8.8.8.8"), 0, false);
        let history = vec![bare(Some("1.1.1.1"), 10, false)];
        let report = evaluate_egress_timeline(&current, &history);
        assert_eq!(report.claim, EgressTimelineClaim::IpChanged);
        assert_eq!(report.previous_ip.as_deref(), Some("1.1.1.1"));
        assert!(report
            .events
            .iter()
            .any(|event| event.kind == EgressTimelineEventKind::IpChanged));
    }

    #[test]
    fn detects_churn() {
        let current = bare(Some("9.9.9.9"), 0, false);
        let history = vec![
            bare(Some("1.1.1.1"), 5, false),
            bare(Some("8.8.8.8"), 10, false),
        ];
        let report = evaluate_egress_timeline(&current, &history);
        assert_eq!(report.claim, EgressTimelineClaim::Churning);
        assert!(report.distinct_ips >= 3);
    }

    #[test]
    fn unstable_current() {
        let current = bare(Some("1.1.1.1"), 0, true);
        let history = vec![bare(Some("1.1.1.1"), 10, false)];
        let report = evaluate_egress_timeline(&current, &history);
        assert_eq!(report.claim, EgressTimelineClaim::Unstable);
    }
}
