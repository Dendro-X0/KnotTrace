//! Path Truth Band 4 — on-device local explain (4A) + cloud prompt helpers (4B).

use crate::diagnostic_bundle::build_diagnostic_bundle;
use crate::integrity_board::INTEGRITY_DISCLAIMER;
use crate::types::*;
use std::collections::BTreeSet;

pub const LOCAL_EXPLAIN_SCHEMA: &str = "knottrace.local_explain.v1";

pub const CLOUD_HARDENING_ADDENDUM: &str = "Reply in markdown with sections: Summary, Likely cause, \
What to do next, What was not checked. Each \"What to do next\" bullet must end with \
(action_id: <id>) using only action_id values present in the bundle recommendations. \
If unsure, say so; do not fabricate probe rows or claim the user is unmonitored.";

pub const CLOUD_USER_PROMPT: &str = "Explain this KnotTrace diagnostic bundle for a non-expert. \
Use only the JSON facts. Map next steps to action_id values from the bundle.";

pub const ALLOWED_ACTION_IDS: &[&str] = &[
    "open_dns_assist",
    "open_connect",
    "open_network",
    "open_protect",
    "review_mtu",
    "review_local_caps",
    "manual_out_of_scope",
];

pub fn build_cloud_system_prompt(bundle: &DiagnosticBundle) -> String {
    format!("{}\n\n{}", bundle.ai_prompt_template, CLOUD_HARDENING_ADDENDUM)
}

/// Extract allowlisted `(action_id: …)` tokens from model markdown.
pub fn parse_cloud_action_ids(markdown: &str) -> (Vec<String>, Vec<String>) {
    let allow: BTreeSet<&str> = ALLOWED_ACTION_IDS.iter().copied().collect();
    let mut parsed = Vec::new();
    let mut rejected = Vec::new();
    let mut seen = BTreeSet::new();

    for raw in extract_action_id_tokens(markdown) {
        let id = raw.trim().to_string();
        if allow.contains(id.as_str()) {
            if seen.insert(id.clone()) {
                parsed.push(id);
            }
        } else if !id.is_empty() && seen.insert(format!("rejected:{id}")) {
            rejected.push(id);
        }
    }
    (parsed, rejected)
}

fn extract_action_id_tokens(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = markdown;
    let mut rest = lower;
    while let Some(start) = rest.find("(action_id:") {
        let after = &rest[start + "(action_id:".len()..];
        if let Some(end) = after.find(')') {
            out.push(after[..end].trim().to_string());
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    out
}

pub fn sanitize_cloud_markdown(input: &str) -> String {
    // Strip crude HTML tags; keep markdown text.
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

pub fn explain_locally(report: &HealthReport) -> LocalExplainReport {
    let bundle = build_diagnostic_bundle(report, DiagnosticBundleOptions::default());
    explain_from_bundle(&bundle, report.integrity.as_ref())
}

pub fn explain_from_bundle(
    bundle: &DiagnosticBundle,
    integrity: Option<&IntegrityBoardReport>,
) -> LocalExplainReport {
    let headline = bundle
        .path_break
        .as_ref()
        .map(|pb| pb.title.clone())
        .or_else(|| bundle.diagnosis.as_ref().map(|d| d.summary.clone()))
        .unwrap_or_else(|| bundle.score.summary.clone());

    let likely_cause = build_likely_cause(bundle);
    let plain_summary = build_plain_summary(bundle, likely_cause.as_deref());
    let next_steps = build_next_steps(bundle);
    let caveats = build_caveats(bundle, integrity);

    LocalExplainReport {
        schema_version: LOCAL_EXPLAIN_SCHEMA.to_string(),
        headline,
        plain_summary,
        likely_cause,
        next_steps,
        caveats,
        source: "local_template".to_string(),
    }
}

fn build_likely_cause(bundle: &DiagnosticBundle) -> Option<String> {
    if let Some(pb) = &bundle.path_break {
        if pb.break_hop != PathBreakHop::None {
            return Some(format!(
                "{} path issue at {} (confidence {:?}).",
                path_label(pb.active_path),
                hop_label(pb.break_hop),
                pb.confidence
            ));
        }
        if !pb.local_ok {
            return Some(format!(
                "Local checks failed before blaming {} egress.",
                path_label(pb.active_path)
            ));
        }
    }
    if let Some(diagnosis) = &bundle.diagnosis {
        if let Some(bottleneck) = diagnosis.primary_bottleneck {
            if bottleneck != BottleneckCategory::Healthy {
                return Some(format!(
                    "Diagnosis points to {:?} (shape {:?}, confidence {:?}).",
                    bottleneck, diagnosis.slowdown_shape, diagnosis.confidence
                ));
            }
        }
        return Some(diagnosis.summary.clone());
    }
    None
}

fn build_plain_summary(bundle: &DiagnosticBundle, likely_cause: Option<&str>) -> String {
    let mut parts = Vec::new();

    parts.push(format!(
        "This check scored {:?} ({}/100): {}.",
        bundle.score.grade, bundle.score.score, bundle.score.summary
    ));

    if let Some(pb) = &bundle.path_break {
        if pb.break_hop == PathBreakHop::None && matches!(bundle.score.grade, HealthGrade::Good) {
            parts.push(
                "No hop failure was classified on the active path; review integrity caveats before assuming everything is fine."
                    .to_string(),
            );
        } else {
            parts.push(pb.summary.clone());
        }
    } else if let Some(diagnosis) = &bundle.diagnosis {
        parts.push(diagnosis.summary.clone());
    }

    if let Some(cause) = likely_cause {
        parts.push(format!("Likely cause reading: {cause}"));
    }

    if bundle.proxy.enabled {
        parts.push("A system proxy is enabled — confirm that matches what you intended.".to_string());
    }
    if bundle.tor.detected && !bundle.tor.socks_reachable {
        parts.push(
            "Tor looks configured but SOCKS is unreachable; treat that as a Tor client/bootstrap issue, not a total internet outage."
                .to_string(),
        );
    }

    parts.join(" ")
}

fn build_next_steps(bundle: &DiagnosticBundle) -> Vec<LocalExplainStep> {
    let mut steps: Vec<LocalExplainStep> = bundle
        .recommendations
        .iter()
        .take(3)
        .map(|rec| LocalExplainStep {
            action_id: rec.action_id.clone(),
            label: rec.title.clone(),
            reason: format!("From recommendation (priority {}).", rec.priority),
        })
        .collect();

    if steps.is_empty() {
        if let Some(pb) = &bundle.path_break {
            if !pb.local_ok || pb.break_hop == PathBreakHop::Dns {
                steps.push(LocalExplainStep {
                    action_id: "open_dns_assist".to_string(),
                    label: "Review DNS Assist".to_string(),
                    reason: "Local/DNS path issues often start with resolver health.".to_string(),
                });
            } else if matches!(
                pb.active_path,
                PathBreakPath::SystemProxy | PathBreakPath::VpnTunnel | PathBreakPath::TorSocks
            ) {
                steps.push(LocalExplainStep {
                    action_id: "open_network".to_string(),
                    label: "Open Network details".to_string(),
                    reason: "Inspect path break, tunnel compare, and integrity indicators.".to_string(),
                });
            }
        }
        if steps.is_empty() {
            steps.push(LocalExplainStep {
                action_id: "open_network".to_string(),
                label: "Review Network page".to_string(),
                reason: "No strong recommendation mapped; inspect measured details.".to_string(),
            });
        }
    }

    steps.truncate(3);
    steps
}

fn build_caveats(
    bundle: &DiagnosticBundle,
    integrity: Option<&IntegrityBoardReport>,
) -> Vec<String> {
    let mut caveats = Vec::new();
    if let Some(board) = integrity {
        if board.level != IntegrityBoardLevel::None {
            caveats.push(INTEGRITY_DISCLAIMER.to_string());
        } else {
            caveats.push(
                "No integrity indicators fired in this check — that is not a clean bill of health."
                    .to_string(),
            );
        }
    } else if !bundle.integrity_flags.is_empty() {
        caveats.push(INTEGRITY_DISCLAIMER.to_string());
    }

    for item in bundle.not_checked.iter().take(4) {
        caveats.push(format!("Not checked: {item}"));
    }
    caveats.push(
        "This explanation is a local template over measured facts — it does not invent probes or apply repairs."
            .to_string(),
    );
    caveats
}

fn path_label(path: PathBreakPath) -> &'static str {
    match path {
        PathBreakPath::Direct => "Direct",
        PathBreakPath::SystemProxy => "System proxy",
        PathBreakPath::VpnTunnel => "VPN",
        PathBreakPath::TorSocks => "Tor SOCKS",
    }
}

fn hop_label(hop: PathBreakHop) -> &'static str {
    match hop {
        PathBreakHop::None => "no break",
        PathBreakHop::Local => "local",
        PathBreakHop::Dns => "DNS",
        PathBreakHop::Tcp => "TCP",
        PathBreakHop::Tls => "TLS",
        PathBreakHop::Http => "HTTP",
        PathBreakHop::Unknown => "unknown hop",
    }
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
                interfaces: vec![],
                dns_servers: vec![],
                proxy: ProxySettings {
                    enabled: false,
                    server: None,
                    source: "none".to_string(),
                },
                tags: vec![EnvironmentTag::HomeLan],
                default_gateway: Some("192.168.1.1".to_string()),
                active_interface: Some("eth0".to_string()),
                tor: default_tor_status(),
            },
            probe: ProbeResult {
                gateway: None,
                internet: Some(LatencySample {
                    target: "1.1.1.1".to_string(),
                    avg_ms: 20.0,
                    loss_pct: 0.0,
                }),
                dns: vec![],
                duration_ms: 1,
            },
            score: HealthScore {
                grade: HealthGrade::Good,
                score: 92,
                summary: "Healthy path".to_string(),
                reasons: vec![],
            },
            dns_integrity: None,
            diagnosis: Some(NetworkDiagnosis {
                summary: "No major bottleneck.".to_string(),
                primary_bottleneck: Some(BottleneckCategory::Healthy),
                slowdown_shape: SlowdownShape::GeneralDegradation,
                confidence: DiagnosisConfidence::Medium,
                hints: vec![],
            }),
            stability: None,
            site_reachability: None,
            egress: None,
            network_context: None,
            recommendations: None,
            proxy_path_report: None,
            link_facts: None,
            local_caps: None,
            mtu_assist: None,
            local_repair: None,
            tunnel_compare: None,
            upstream_pool: None,
            path_break: Some(PathBreakReport {
                local_ok: true,
                local_issues: vec![],
                active_path: PathBreakPath::Direct,
                paths_present: vec![PathBreakPath::Direct],
                break_hop: PathBreakHop::None,
                environment_class: PathEnvironmentClass::HomeLan,
                confidence: PathBreakConfidence::Medium,
                title: "Direct path looks reachable".to_string(),
                summary: "Active path: Direct.".to_string(),
                evidence: vec![],
                not_checked: vec![],
            }),
            integrity: Some(IntegrityBoardReport {
                indicator_count: 0,
                level: IntegrityBoardLevel::None,
                title: "No integrity indicators in this check".to_string(),
                summary: "none".to_string(),
                disclaimer: INTEGRITY_DISCLAIMER.to_string(),
                indicators: vec![],
                listeners: None,
                gateway_arp: None,
                cert_store: None,
                process_egress: None,
                autoruns: None,
                not_checked: vec![],
                confidence: IntegrityBoardConfidence::Low,
            }),
            egress_timeline: None,
        }
    }

    #[test]
    fn healthy_local_explain_avoids_safe_claims() {
        let explain = explain_locally(&base_report());
        assert_eq!(explain.schema_version, LOCAL_EXPLAIN_SCHEMA);
        assert_eq!(explain.source, "local_template");
        assert!(explain.headline.contains("Direct"));
        assert!(explain.plain_summary.contains("No hop failure"));
        assert!(!explain.plain_summary.to_lowercase().contains("boost"));
        assert!(!explain.next_steps.is_empty());
        assert!(explain
            .caveats
            .iter()
            .any(|c| c.contains("not a clean bill") || c.contains("Not checked")));
    }

    #[test]
    fn proxy_tls_lists_cause_and_actions() {
        let mut report = base_report();
        report.environment.proxy.enabled = true;
        report.score.grade = HealthGrade::Poor;
        report.score.score = 40;
        report.path_break = Some(PathBreakReport {
            local_ok: true,
            local_issues: vec![],
            active_path: PathBreakPath::SystemProxy,
            paths_present: vec![PathBreakPath::Direct, PathBreakPath::SystemProxy],
            break_hop: PathBreakHop::Tls,
            environment_class: PathEnvironmentClass::CorporateProxy,
            confidence: PathBreakConfidence::High,
            title: "System proxy path fails at TLS".to_string(),
            summary: "Break at TLS.".to_string(),
            evidence: vec!["tls".to_string()],
            not_checked: vec![],
        });
        report.recommendations = Some(NetworkRecommendations {
            items: vec![NetworkRecommendation {
                category: RecommendationCategory::ProxyPath,
                priority: 1,
                title: "Check proxy path".to_string(),
                message: "TLS failures on proxy.".to_string(),
            }],
            summary: "proxy".to_string(),
        });
        let explain = explain_locally(&report);
        assert!(explain.headline.contains("TLS"));
        assert!(explain
            .likely_cause
            .as_ref()
            .is_some_and(|c| c.contains("TLS")));
        assert_eq!(explain.next_steps[0].action_id, "open_connect");
    }

    #[test]
    fn tor_socks_down_mentions_bootstrap() {
        let mut report = base_report();
        report.environment.tor = TorStatus {
            detected: true,
            socks_endpoint: Some("127.0.0.1:9050".to_string()),
            socks_reachable: false,
            source: "test".to_string(),
        };
        report.path_break = Some(PathBreakReport {
            local_ok: false,
            local_issues: vec!["socks".to_string()],
            active_path: PathBreakPath::TorSocks,
            paths_present: vec![PathBreakPath::Direct, PathBreakPath::TorSocks],
            break_hop: PathBreakHop::Local,
            environment_class: PathEnvironmentClass::Tor,
            confidence: PathBreakConfidence::High,
            title: "Local path issue before Tor SOCKS egress".to_string(),
            summary: "SOCKS down.".to_string(),
            evidence: vec![],
            not_checked: vec![],
        });
        let explain = explain_locally(&report);
        assert!(explain.plain_summary.contains("SOCKS"));
        assert!(explain
            .likely_cause
            .as_ref()
            .is_some_and(|c| c.contains("Local") || c.contains("Tor")));
    }

    #[test]
    fn integrity_elevated_adds_disclaimer() {
        let mut report = base_report();
        report.integrity = Some(IntegrityBoardReport {
            indicator_count: 1,
            level: IntegrityBoardLevel::Elevated,
            title: "1 indicator".to_string(),
            summary: "dns".to_string(),
            disclaimer: INTEGRITY_DISCLAIMER.to_string(),
            indicators: vec![],
            listeners: None,
            gateway_arp: None,
            cert_store: None,
            process_egress: None,
                autoruns: None,
            not_checked: vec![],
            confidence: IntegrityBoardConfidence::High,
        });
        let explain = explain_locally(&report);
        assert!(explain
            .caveats
            .iter()
            .any(|c| c.contains("does not mean you are unobserved")));
    }

    #[test]
    fn parses_allowlisted_action_ids() {
        let md = "### What to do next\n- Check DNS (action_id: open_dns_assist)\n- Hack (action_id: erase_disk)\n";
        let (parsed, rejected) = parse_cloud_action_ids(md);
        assert_eq!(parsed, vec!["open_dns_assist".to_string()]);
        assert_eq!(rejected, vec!["erase_disk".to_string()]);
    }

    #[test]
    fn sanitizes_html_from_cloud_markdown() {
        let clean = sanitize_cloud_markdown("Hello <script>x</script> world");
        assert!(!clean.contains('<'));
        assert!(!clean.contains('>'));
        assert!(clean.contains("Hello"));
        assert!(clean.contains("world"));
    }
}
