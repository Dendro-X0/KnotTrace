use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

const MIN_CHECKS_FOR_RECURRENCE: usize = 3;
const MIN_IMPAIRED_FOR_RECURRENCE: usize = 2;
const MIN_SPAN_MINUTES_FOR_POOL: i64 = 15;
const HISTORY_WINDOW: usize = 48;

pub fn evaluate_upstream_pool_proof(
    current: &HealthReport,
    history: &[HealthReport],
) -> Option<UpstreamPoolProof> {
    if !current.environment.proxy.enabled {
        return Some(UpstreamPoolProof {
            claim: UpstreamPoolClaim::None,
            confidence: UpstreamPoolConfidence::High,
            title: "No upstream pool claim".to_string(),
            summary: "System proxy is off — KnotTrace is not blaming an upstream proxy pool.".to_string(),
            evidence: Vec::new(),
            not_proven: vec!["Upstream pool quality (proxy is off).".to_string()],
            action: "No proxy-pool action needed.".to_string(),
            intermittent_domains: Vec::new(),
            recurring_impaired_checks: 0,
            distinct_egress_ips: 0,
            proxy_only_failure_domains: Vec::new(),
            checks_considered: 0,
        });
    }

    let snapshot = snapshot_assessment(current);
    let mut series: Vec<&HealthReport> = history
        .iter()
        .filter(|report| report.environment.proxy.enabled)
        .collect();
    if !series.iter().any(|report| report.timestamp == current.timestamp) {
        series.insert(0, current);
    }
    series.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
    series.truncate(HISTORY_WINDOW);

    let intermittent_domains = detect_intermittent_domains(&series);
    let impaired_reports: Vec<&&HealthReport> = series
        .iter()
        .filter(|report| snapshot_is_impaired(report))
        .collect();
    let recurring_impaired_checks = impaired_reports.len() as u8;
    let distinct_egress_ips = distinct_egress_count(&impaired_reports);
    let checks_considered = series.len() as u8;
    let span_ok = time_span_minutes(&series) >= MIN_SPAN_MINUTES_FOR_POOL
        || series.len() >= MIN_CHECKS_FOR_RECURRENCE + 2;

    let proxy_only_failure_domains = current
        .proxy_path_report
        .as_ref()
        .map(|path| {
            path.comparisons
                .iter()
                .filter(|row| row.proxy_only_failure)
                .map(|row| row.domain.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let (claim, confidence, title, summary, evidence, not_proven, action) = if impaired_reports
        .len()
        >= MIN_IMPAIRED_FOR_RECURRENCE
        && series.len() >= MIN_CHECKS_FOR_RECURRENCE
        && distinct_egress_ips >= 2
        && span_ok
    {
        (
            UpstreamPoolClaim::UpstreamPoolPoor,
            UpstreamPoolConfidence::High,
            "Upstream pool quality looks poor".to_string(),
            "Proxy-only site failures recur across multiple checks and distinct egress IPs. The bottleneck is upstream — do not thrash nodes hunting for speed.".to_string(),
            vec![
                format!(
                    "{recurring_impaired_checks} impaired checks with ≥2 proxy-only failures while direct stayed healthier."
                ),
                format!("{distinct_egress_ips} distinct public egress IPs showed the pattern."),
                format!("Checks considered: {checks_considered}."),
            ],
            vec![
                "Seller rate limits / Mbps (reachability proof only).".to_string(),
                "Whole subscription inventory (only sampled exits seen by this device).".to_string(),
            ],
            "Change provider or stop sending critical traffic through this pool. Avoid rapid node/IP rotation.".to_string(),
        )
    } else if impaired_reports.len() >= MIN_IMPAIRED_FOR_RECURRENCE
        && series.len() >= MIN_CHECKS_FOR_RECURRENCE
    {
        (
            UpstreamPoolClaim::ActivePathRecurring,
            UpstreamPoolConfidence::High,
            "Active proxy path is recurring impaired".to_string(),
            "The same proxy-only failure pattern repeats over time. Pool-wide quality is suspected but not fully proven (limited exit diversity).".to_string(),
            vec![
                format!("{recurring_impaired_checks} recurring impaired checks while proxy stayed on."),
                format!("Distinct egress IPs in impaired set: {distinct_egress_ips}."),
            ],
            vec![
                "Whole upstream pool (need ≥2 distinct egress IPs with the same pattern).".to_string(),
            ],
            "Do not rapidly switch nodes. Prefer one careful change, split-tunnel critical apps, or change provider.".to_string(),
        )
    } else if snapshot.impaired {
        let confidence = if snapshot.proxy_only_failures >= 2 {
            UpstreamPoolConfidence::High
        } else {
            UpstreamPoolConfidence::Medium
        };
        (
            UpstreamPoolClaim::ActivePathImpaired,
            confidence,
            "Active proxy path is impaired".to_string(),
            "Sites failed on the proxy path but succeeded on direct in this check. That proves the current route is bad — not yet that the whole pool is bad.".to_string(),
            snapshot.evidence,
            vec![
                "Upstream pool quality (need recurrence across checks / exits).".to_string(),
            ],
            "Optional: try one other node once via Connect Assist. Avoid thrashing exits.".to_string(),
        )
    } else if !intermittent_domains.is_empty() {
        (
            UpstreamPoolClaim::ActivePathImpaired,
            UpstreamPoolConfidence::Medium,
            "Intermittent proxy-only site failures".to_string(),
            format!(
                "Sensitive hosts flip between OK and fail on the proxy only ({}). Classic low-quality or overloaded upstream filtering — not a total local outage.",
                intermittent_domains.join(", ")
            ),
            vec![format!(
                "Intermittent proxy-only domains: {}.",
                intermittent_domains.join(", ")
            )],
            vec!["Stable whole-pool failure (pattern is intermittent).".to_string()],
            "Prefer split-tunnel for git/mail if possible; avoid rapid IP rotation.".to_string(),
        )
    } else {
        (
            UpstreamPoolClaim::Inconclusive,
            UpstreamPoolConfidence::Low,
            "Upstream pool evidence inconclusive".to_string(),
            snapshot.summary,
            snapshot.evidence,
            vec![
                "Active path impairment (≥2 proxy-only failures with direct OK).".to_string(),
                "Recurring pool-wide pattern.".to_string(),
            ],
            "Keep observing. Do not assume the provider pool is fine or broken yet.".to_string(),
        )
    };

    let mut evidence = evidence;
    if !proxy_only_failure_domains.is_empty()
        && !evidence
            .iter()
            .any(|item| item.contains("Current proxy-only failures"))
    {
        evidence.push(format!(
            "Current proxy-only failures: {}.",
            proxy_only_failure_domains.join(", ")
        ));
    }
    if !intermittent_domains.is_empty()
        && !evidence
            .iter()
            .any(|item| item.to_ascii_lowercase().contains("intermittent"))
    {
        evidence.push(format!(
            "Intermittent proxy-only domains: {}.",
            intermittent_domains.join(", ")
        ));
    }

    Some(UpstreamPoolProof {
        claim,
        confidence,
        title,
        summary,
        evidence,
        not_proven,
        action,
        intermittent_domains,
        recurring_impaired_checks,
        distinct_egress_ips,
        proxy_only_failure_domains,
        checks_considered,
    })
}

struct SnapshotAssessment {
    impaired: bool,
    proxy_only_failures: u8,
    evidence: Vec<String>,
    summary: String,
}

fn snapshot_assessment(report: &HealthReport) -> SnapshotAssessment {
    let Some(path) = &report.proxy_path_report else {
        return SnapshotAssessment {
            impaired: false,
            proxy_only_failures: 0,
            evidence: vec!["No proxy path report in this check.".to_string()],
            summary: "Proxy is on, but no differential proxy-vs-direct site report is available yet."
                .to_string(),
        };
    };

    let impaired = path.proxy_only_failure_count >= 1 && path.direct_failure_count == 0;

    let evidence = vec![format!(
        "Snapshot: {} proxy-only / {} proxy / {} direct failures across {} sites ({:?} confidence).",
        path.proxy_only_failure_count,
        path.proxy_failure_count,
        path.direct_failure_count,
        path.checked_domains,
        path.confidence
    )];

    SnapshotAssessment {
        impaired,
        proxy_only_failures: path.proxy_only_failure_count,
        evidence,
        summary: path.summary.clone(),
    }
}

fn snapshot_is_impaired(report: &HealthReport) -> bool {
    report.proxy_path_report.as_ref().is_some_and(|path| {
        path.proxy_only_failure_count >= 2 && path.direct_failure_count == 0
    })
}

fn distinct_egress_count(reports: &[&&HealthReport]) -> u8 {
    let mut ips = BTreeSet::new();
    for report in reports {
        if let Some(ip) = report.egress.as_ref().and_then(|egress| egress.primary_ip.clone()) {
            ips.insert(ip);
        }
    }
    ips.len() as u8
}

fn time_span_minutes(reports: &[&HealthReport]) -> i64 {
    let Some(newest) = reports.first() else {
        return 0;
    };
    let Some(oldest) = reports.last() else {
        return 0;
    };
    newest
        .timestamp
        .signed_duration_since(oldest.timestamp)
        .num_minutes()
        .abs()
}

fn detect_intermittent_domains(reports: &[&HealthReport]) -> Vec<String> {
    let mut proxy_ok: BTreeMap<String, u8> = BTreeMap::new();
    let mut proxy_fail: BTreeMap<String, u8> = BTreeMap::new();
    let mut direct_fail: BTreeMap<String, u8> = BTreeMap::new();

    for report in reports {
        let Some(path) = &report.proxy_path_report else {
            continue;
        };
        for row in &path.comparisons {
            if row.proxy.success {
                *proxy_ok.entry(row.domain.clone()).or_default() += 1;
            } else {
                *proxy_fail.entry(row.domain.clone()).or_default() += 1;
            }
            if !row.direct.success {
                *direct_fail.entry(row.domain.clone()).or_default() += 1;
            }
        }
    }

    let mut domains = Vec::new();
    for (domain, fails) in proxy_fail {
        let oks = proxy_ok.get(&domain).copied().unwrap_or(0);
        let direct_fails = direct_fail.get(&domain).copied().unwrap_or(0);
        if oks >= 1 && fails >= 1 && direct_fails == 0 {
            domains.push(domain);
        }
    }
    domains.sort();
    domains
}

/// Convenience for tests: evaluate using only the current report as history.
pub fn evaluate_upstream_pool_snapshot(current: &HealthReport) -> Option<UpstreamPoolProof> {
    evaluate_upstream_pool_proof(current, &[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration as ChronoDuration, Utc};

    fn base_report(proxy_on: bool) -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "host".to_string(),
                interfaces: Vec::new(),
                dns_servers: Vec::new(),
                proxy: ProxySettings {
                    enabled: proxy_on,
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
                    target: "1.1.1.1".to_string(),
                    avg_ms: 30.0,
                    loss_pct: 0.0,
                }),
                dns: Vec::new(),
                duration_ms: 10,
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
            site_reachability: None,
            egress: Some(EgressReport {
                primary_ip: Some("1.1.1.1".to_string()),
                confidence: EgressConfidence::High,
                system_path: EgressPathReport {
                    kind: EgressPathKind::System,
                    primary_ip: Some("1.1.1.1".to_string()),
                    endpoints: Vec::new(),
                    summary: "ok".to_string(),
                },
                tor_path: None,
                summary: "ok".to_string(),
            }),
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

    fn comparison(domain: &str, proxy_ok: bool, direct_ok: bool) -> ProxyPathDomainComparison {
        ProxyPathDomainComparison {
            domain: domain.to_string(),
            proxy: SiteReachResult {
                domain: domain.to_string(),
                success: proxy_ok,
                status_code: if proxy_ok { Some(200) } else { None },
                latency_ms: Some(100.0),
                error: if proxy_ok {
                    None
                } else {
                    Some("timeout".to_string())
                },
                error_kind: if proxy_ok {
                    None
                } else {
                    Some(SiteReachErrorKind::Timeout)
                },
            },
            direct: SiteReachResult {
                domain: domain.to_string(),
                success: direct_ok,
                status_code: if direct_ok { Some(200) } else { None },
                latency_ms: Some(40.0),
                error: None,
                error_kind: None,
            },
            proxy_only_failure: !proxy_ok && direct_ok,
        }
    }

    fn path_report(rows: Vec<ProxyPathDomainComparison>) -> ProxyPathReport {
        let proxy_only = rows.iter().filter(|row| row.proxy_only_failure).count() as u8;
        let proxy_fail = rows.iter().filter(|row| !row.proxy.success).count() as u8;
        let direct_fail = rows.iter().filter(|row| !row.direct.success).count() as u8;
        ProxyPathReport {
            checked_domains: rows.len() as u8,
            proxy_failure_count: proxy_fail,
            direct_failure_count: direct_fail,
            proxy_only_failure_count: proxy_only,
            confidence: if proxy_only >= 2 {
                ProxyPathConfidence::High
            } else {
                ProxyPathConfidence::Medium
            },
            likely_provider_side: proxy_only > 0 && direct_fail == 0,
            summary: "test path".to_string(),
            comparisons: rows,
        }
    }

    #[test]
    fn proxy_off_yields_none_claim() {
        let report = base_report(false);
        let proof = evaluate_upstream_pool_snapshot(&report).unwrap();
        assert_eq!(proof.claim, UpstreamPoolClaim::None);
    }

    #[test]
    fn first_impaired_snapshot_is_active_path_not_pool() {
        let mut report = base_report(true);
        report.proxy_path_report = Some(path_report(vec![
            comparison("github.com", false, true),
            comparison("www.google.com", false, true),
            comparison("www.cloudflare.com", true, true),
        ]));
        let proof = evaluate_upstream_pool_snapshot(&report).unwrap();
        assert_eq!(proof.claim, UpstreamPoolClaim::ActivePathImpaired);
        assert_ne!(proof.claim, UpstreamPoolClaim::UpstreamPoolPoor);
        assert!(proof.action.to_ascii_lowercase().contains("avoid") || proof.action.contains("once"));
    }

    #[test]
    fn recurrence_across_egress_ips_marks_pool_poor() {
        let mut reports = Vec::new();
        for (idx, ip) in ["10.0.0.1", "10.0.0.2", "10.0.0.1"].iter().enumerate() {
            let mut report = base_report(true);
            report.timestamp = Utc::now() - ChronoDuration::minutes(10 * idx as i64);
            report.egress.as_mut().unwrap().primary_ip = Some((*ip).to_string());
            report.proxy_path_report = Some(path_report(vec![
                comparison("github.com", false, true),
                comparison("api.github.com", false, true),
            ]));
            reports.push(report);
        }

        let proof = evaluate_upstream_pool_proof(&reports[0], &reports[1..]).unwrap();
        assert_eq!(proof.claim, UpstreamPoolClaim::UpstreamPoolPoor);
        assert_eq!(proof.confidence, UpstreamPoolConfidence::High);
        assert!(proof.action.to_ascii_lowercase().contains("provider") || proof.action.contains("pool"));
        assert!(!proof.action.to_ascii_lowercase().contains("switch nodes rapidly"));
    }

    #[test]
    fn intermittent_domain_detection() {
        let mut ok = base_report(true);
        ok.timestamp = Utc::now() - ChronoDuration::minutes(5);
        ok.proxy_path_report = Some(path_report(vec![comparison("github.com", true, true)]));

        let mut fail = base_report(true);
        fail.proxy_path_report = Some(path_report(vec![comparison("github.com", false, true)]));

        let proof = evaluate_upstream_pool_proof(&fail, &[ok]).unwrap();
        assert!(proof.intermittent_domains.iter().any(|domain| domain == "github.com"));
        assert_eq!(proof.claim, UpstreamPoolClaim::ActivePathImpaired);
    }
}
