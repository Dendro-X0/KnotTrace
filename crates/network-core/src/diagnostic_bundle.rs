//! Redacted diagnostic bundle for Path Truth Band 2 (AI-ready JSON, no cloud).

use crate::assist::load_backup as load_dns_backup;
use crate::auto_protect_log::list_auto_protect_log;
use crate::local_caps::load_backup as load_local_caps_backup;
use crate::mtu_assist::load_backup as load_mtu_backup;
use crate::types::*;
use chrono::{DateTime, Utc};
use std::path::Path;

pub const AI_PROMPT_TEMPLATE: &str = "You are helping a user understand KnotTrace network diagnostics. \
Use ONLY the structured facts in the attached JSON bundle. Do not invent probe results, scores, \
or hop failures. Map every suggestion to an action_id from the bundle recommendations, or say \
manual / out of scope. Never claim the user is unmonitored or guarantee Mbps improvements. \
Absence of integrity indicators does not mean the path is safe from surveillance.";

const NOT_CHECKED: &[&str] = &[
    "Listening services / high-risk ports (Band 3)",
    "Process ↔ egress correlation (Protect opt-in; omitted unless enabled on device)",
    "SSID (omitted unless include_ssid)",
    "Hostname (always omitted from export)",
    "Cloud AI transfer (Band 4 — not used by this export)",
];

pub fn build_diagnostic_bundle(
    report: &HealthReport,
    options: DiagnosticBundleOptions,
) -> DiagnosticBundle {
    build_diagnostic_bundle_with_dir(report, options, None)
}

pub fn build_diagnostic_bundle_with_dir(
    report: &HealthReport,
    options: DiagnosticBundleOptions,
    data_dir: Option<&Path>,
) -> DiagnosticBundle {
    let mut redaction_notes = vec![
        "Hostname omitted.".to_string(),
        "Full HealthReport not included — curated fields only.".to_string(),
    ];
    if !options.include_ssid {
        redaction_notes.push("SSID omitted (default).".to_string());
    } else {
        redaction_notes.push("SSID inclusion requested; Wi‑Fi SSID may appear in link summary if present.".to_string());
    }

    let path_break = report.path_break.as_ref().map(|pb| DiagnosticPathBreakSummary {
        local_ok: pb.local_ok,
        active_path: pb.active_path,
        break_hop: pb.break_hop,
        environment_class: pb.environment_class,
        confidence: pb.confidence,
        title: pb.title.clone(),
        summary: pb.summary.clone(),
    });

    let mut not_checked: Vec<String> = NOT_CHECKED.iter().map(|s| (*s).to_string()).collect();
    if let Some(pb) = &report.path_break {
        for item in &pb.not_checked {
            if !not_checked.iter().any(|existing| existing == item) {
                not_checked.push(item.clone());
            }
        }
    }
    if let Some(board) = &report.integrity {
        for item in &board.not_checked {
            if !not_checked.iter().any(|existing| existing == item) {
                not_checked.push(item.clone());
            }
        }
        // Drop legacy Band 3 ARP/cert honesty lines when snapshots observed.
        if board.gateway_arp.as_ref().is_some_and(|s| s.observed) {
            not_checked.retain(|s| !s.contains("ARP"));
        }
        if board.cert_store.as_ref().is_some_and(|s| s.observed) {
            not_checked.retain(|s| !s.contains("Trusted root") && !s.contains("certificate delta"));
        }
    } else {
        not_checked.push("Trusted root certificate deltas (Band 3)".to_string());
        not_checked.push("Gateway / ARP identity churn (Band 3)".to_string());
    }

    DiagnosticBundle {
        schema_version: DIAGNOSTIC_BUNDLE_SCHEMA.to_string(),
        exported_at: Utc::now(),
        app: DiagnosticBundleApp {
            name: "KnotTrace".to_string(),
            version: option_env!("CARGO_PKG_VERSION").map(|v| v.to_string()),
        },
        redaction: DiagnosticBundleRedaction {
            include_ssid: options.include_ssid,
            notes: redaction_notes,
        },
        os: std::env::consts::OS.to_string(),
        environment_class: report
            .path_break
            .as_ref()
            .map(|pb| pb.environment_class),
        path_break,
        score: DiagnosticScoreSummary {
            grade: report.score.grade,
            score: report.score.score,
            summary: report.score.summary.clone(),
        },
        diagnosis: report.diagnosis.as_ref().map(|d| DiagnosticDiagnosisSummary {
            slowdown_shape: d.slowdown_shape,
            confidence: d.confidence,
            primary_bottleneck: d.primary_bottleneck,
            summary: d.summary.clone(),
        }),
        network_context: report.network_context.as_ref().map(|ctx| DiagnosticContextSummary {
            kind: ctx.kind,
            risk_level: ctx.risk_level,
            captive_state: ctx.captive_portal.state,
            summary: ctx.summary.clone(),
        }),
        dns_integrity: report.dns_integrity.as_ref().map(|dns| DiagnosticDnsIntegritySummary {
            state: dns.state,
            confidence: dns.confidence,
            mismatch_count: dns.mismatch_count,
            summary: dns.summary.clone(),
        }),
        proxy: DiagnosticProxySummary {
            enabled: report.environment.proxy.enabled,
            server: report.environment.proxy.server.clone(),
        },
        tor: DiagnosticTorSummary {
            detected: report.environment.tor.detected
                || report.environment.tags.contains(&EnvironmentTag::Tor),
            socks_reachable: report.environment.tor.socks_reachable,
        },
        vpn_detected: report.environment.tags.contains(&EnvironmentTag::Vpn),
        probe_table: build_probe_table(report),
        link_summary: link_summary(report, options.include_ssid),
        upstream_pool: report.upstream_pool.as_ref().map(|pool| DiagnosticUpstreamSummary {
            claim: pool.claim,
            confidence: pool.confidence,
            title: pool.title.clone(),
        }),
        recommendations: map_recommendations(report),
        integrity_flags: collect_integrity_flags(report),
        config_timeline: data_dir
            .map(build_config_timeline)
            .unwrap_or_default(),
        not_checked,
        ai_prompt_template: AI_PROMPT_TEMPLATE.to_string(),
    }
}

/// Local config-change timeline from assist backups + recent auto-protect log.
pub fn build_config_timeline(data_dir: &Path) -> Vec<String> {
    let mut events: Vec<(DateTime<Utc>, String)> = Vec::new();

    if let Ok(Some(backup)) = load_dns_backup(data_dir) {
        events.push((
            backup.applied_at,
            format!(
                "DNS Assist applied on {} → {} (restore available)",
                backup.interface_alias, backup.applied_resolver
            ),
        ));
    }
    if let Ok(Some(backup)) = load_local_caps_backup(data_dir) {
        events.push((
            backup.applied_at,
            format!(
                "Local caps repair applied (autotuning was {})",
                backup.previous_autotuning_level
            ),
        ));
    }
    if let Ok(Some(backup)) = load_mtu_backup(data_dir) {
        events.push((
            backup.applied_at,
            format!(
                "MTU assist on {} {}→{} (restore available)",
                backup.interface_name, backup.previous_mtu, backup.applied_mtu
            ),
        ));
    }
    if let Ok(entries) = list_auto_protect_log(data_dir, 12) {
        for entry in entries {
            let status = if entry.success { "ok" } else { "skipped" };
            events.push((
                entry.timestamp,
                format!(
                    "Auto-protect {}: {} ({})",
                    entry.kind, entry.message, status
                ),
            ));
        }
    }

    events.sort_by(|a, b| b.0.cmp(&a.0));
    events
        .into_iter()
        .take(12)
        .map(|(at, detail)| format!("{} · {}", at.to_rfc3339(), detail))
        .collect()
}

fn action_id_for(category: RecommendationCategory) -> &'static str {
    match category {
        RecommendationCategory::DnsSecurity => "open_dns_assist",
        RecommendationCategory::ProxyPath => "open_connect",
        RecommendationCategory::TorPath | RecommendationCategory::VpnPrivacy => "open_network",
        RecommendationCategory::PublicNetwork | RecommendationCategory::CaptivePortal => {
            "open_protect"
        }
        RecommendationCategory::LinkLocal => "open_network",
        RecommendationCategory::Egress => "open_network",
        RecommendationCategory::General => "manual_out_of_scope",
    }
}

fn map_recommendations(report: &HealthReport) -> Vec<DiagnosticRecommendation> {
    report
        .recommendations
        .as_ref()
        .map(|recs| {
            recs.items
                .iter()
                .map(|item| DiagnosticRecommendation {
                    title: item.title.clone(),
                    category: item.category,
                    priority: item.priority,
                    action_id: action_id_for(item.category).to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn collect_integrity_flags(report: &HealthReport) -> Vec<String> {
    if let Some(board) = &report.integrity {
        if !board.indicators.is_empty() {
            return board
                .indicators
                .iter()
                .map(|indicator| format!("{}: {}", indicator.title, indicator.message))
                .collect();
        }
    }

    let mut flags = Vec::new();
    if report.environment.proxy.enabled {
        flags.push("System / HTTP(S) proxy enabled.".to_string());
    }
    if let Some(dns) = &report.dns_integrity {
        match dns.state {
            DnsIntegrityState::Suspicious => {
                flags.push(format!("DNS integrity suspicious: {}", dns.summary));
            }
            DnsIntegrityState::Caution => {
                flags.push(format!("DNS integrity caution: {}", dns.summary));
            }
            DnsIntegrityState::Ok => {}
        }
    }
    if let Some(ctx) = &report.network_context {
        match ctx.captive_portal.state {
            CaptivePortalState::Confirmed => {
                flags.push("Captive portal confirmed.".to_string());
            }
            CaptivePortalState::Suspected => {
                flags.push("Captive portal suspected.".to_string());
            }
            CaptivePortalState::NotDetected => {}
        }
    }
    flags
}

fn link_summary(report: &HealthReport, include_ssid: bool) -> Option<String> {
    let facts = report.link_facts.as_ref()?;
    let mut parts = Vec::new();
    if let Some(active) = &facts.active {
        parts.push(format!(
            "active={} kind={:?} up={}",
            active.friendly_name.as_ref().unwrap_or(&active.name),
            active.kind,
            active.is_up
        ));
        if let Some(speed) = active.speed_mbps {
            parts.push(format!("speed_mbps={speed}"));
        }
        if include_ssid {
            if let Some(ssid) = &active.wifi_ssid {
                parts.push(format!("ssid={ssid}"));
            }
        }
    }
    if parts.is_empty() {
        Some(facts.summary.clone())
    } else {
        Some(parts.join("; "))
    }
}

fn build_probe_table(report: &HealthReport) -> Vec<DiagnosticProbeRow> {
    let mut rows = Vec::new();

    if let Some(gateway) = &report.probe.gateway {
        rows.push(DiagnosticProbeRow {
            path: "system".to_string(),
            stage: "local".to_string(),
            ok: gateway.loss_pct < 100.0,
            detail: format!(
                "gateway {} avg_ms={:.0} loss={:.0}%",
                gateway.target, gateway.avg_ms, gateway.loss_pct
            ),
        });
    }

    for dns in &report.probe.dns {
        rows.push(DiagnosticProbeRow {
            path: "system".to_string(),
            stage: "dns".to_string(),
            ok: dns.success,
            detail: format!(
                "resolver={} query={} latency_ms={:.0}",
                dns.resolver, dns.query, dns.latency_ms
            ),
        });
    }

    if let Some(internet) = &report.probe.internet {
        rows.push(DiagnosticProbeRow {
            path: "system".to_string(),
            stage: "tcp".to_string(),
            ok: internet.loss_pct < 100.0,
            detail: format!(
                "internet {} avg_ms={:.0} loss={:.0}%",
                internet.target, internet.avg_ms, internet.loss_pct
            ),
        });
    }

    if let Some(reach) = &report.site_reachability {
        for result in &reach.results {
            rows.push(DiagnosticProbeRow {
                path: "system".to_string(),
                stage: stage_from_error(result.error_kind),
                ok: result.success,
                detail: format!(
                    "{}{}",
                    result.domain,
                    result
                        .error_kind
                        .map(|k| format!(" ({k:?})"))
                        .unwrap_or_default()
                ),
            });
        }
    }

    if let Some(compare) = &report.tunnel_compare {
        for sample in &compare.paths {
            let path = match sample.kind {
                TunnelPathKind::Direct => "direct",
                TunnelPathKind::SystemProxy => "system_proxy",
                TunnelPathKind::TorSocks => "tor",
            };
            rows.push(DiagnosticProbeRow {
                path: path.to_string(),
                stage: "reachability".to_string(),
                ok: sample.available && sample.failure_count == 0,
                detail: format!(
                    "{} success={} failure={}{}",
                    sample.label,
                    sample.success_count,
                    sample.failure_count,
                    sample
                        .median_latency_ms
                        .map(|ms| format!(" median_ms={ms:.0}"))
                        .unwrap_or_default()
                ),
            });
        }
    }

    rows
}

fn stage_from_error(kind: Option<SiteReachErrorKind>) -> String {
    match kind {
        Some(SiteReachErrorKind::Dns) => "dns",
        Some(SiteReachErrorKind::Timeout) | Some(SiteReachErrorKind::ConnectionReset) => "tcp",
        Some(SiteReachErrorKind::Tls) => "tls",
        Some(SiteReachErrorKind::HttpServer) | Some(SiteReachErrorKind::HttpBlocked) => "http",
        Some(SiteReachErrorKind::Proxy) => "tcp",
        Some(SiteReachErrorKind::Unknown) | None => "reachability",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_report() -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
            environment: EnvironmentSnapshot {
                hostname: "SECRET-HOST".to_string(),
                interfaces: vec![],
                dns_servers: vec![],
                proxy: ProxySettings {
                    enabled: true,
                    server: Some("127.0.0.1:7890".to_string()),
                    source: "system".to_string(),
                },
                tags: vec![EnvironmentTag::Proxy],
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
                dns: vec![DnsProbe {
                    resolver: "1.1.1.1".to_string(),
                    query: "example.com".to_string(),
                    latency_ms: 12.0,
                    success: true,
                }],
                duration_ms: 1,
            },
            score: HealthScore {
                grade: HealthGrade::Fair,
                score: 70,
                summary: "fair".to_string(),
                reasons: vec![],
            },
            dns_integrity: Some(DnsIntegrityStatus {
                state: DnsIntegrityState::Caution,
                confidence: DnsIntegrityConfidence::Medium,
                mismatch_count: 1,
                checked_domains: 3,
                summary: "One mismatch.".to_string(),
                details: vec![],
            }),
            diagnosis: None,
            stability: None,
            site_reachability: None,
            egress: None,
            network_context: None,
            recommendations: Some(NetworkRecommendations {
                items: vec![NetworkRecommendation {
                    category: RecommendationCategory::DnsSecurity,
                    priority: 1,
                    title: "Review DNS".to_string(),
                    message: "Integrity caution.".to_string(),
                }],
                summary: "dns".to_string(),
            }),
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
                active_path: PathBreakPath::SystemProxy,
                paths_present: vec![PathBreakPath::Direct, PathBreakPath::SystemProxy],
                break_hop: PathBreakHop::None,
                environment_class: PathEnvironmentClass::CorporateProxy,
                confidence: PathBreakConfidence::Medium,
                title: "System proxy path looks reachable".to_string(),
                summary: "ok".to_string(),
                evidence: vec![],
                not_checked: vec!["listeners".to_string()],
            }),
            integrity: None,
            egress_timeline: None,
        }
    }

    #[test]
    fn bundle_omits_hostname_and_maps_actions() {
        let bundle = build_diagnostic_bundle(&sample_report(), DiagnosticBundleOptions::default());
        let json = serde_json::to_string(&bundle).expect("serialize");
        assert!(!json.contains("SECRET-HOST"));
        assert_eq!(bundle.schema_version, DIAGNOSTIC_BUNDLE_SCHEMA);
        assert!(!bundle.ai_prompt_template.is_empty());
        assert!(!bundle.not_checked.is_empty());
        assert_eq!(bundle.recommendations[0].action_id, "open_dns_assist");
        assert!(bundle
            .integrity_flags
            .iter()
            .any(|f| f.contains("proxy enabled")));
        assert_eq!(
            bundle.environment_class,
            Some(PathEnvironmentClass::CorporateProxy)
        );
    }

    #[test]
    fn probe_table_includes_dns_row() {
        let bundle = build_diagnostic_bundle(&sample_report(), DiagnosticBundleOptions::default());
        assert!(bundle.probe_table.iter().any(|row| row.stage == "dns"));
    }

    #[test]
    fn config_timeline_reads_dns_backup() {
        let dir = std::env::temp_dir().join(format!(
            "knottrace-timeline-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let backup = DnsBackup {
            interface_alias: "Ethernet".to_string(),
            previous_servers: vec!["192.168.1.1".to_string()],
            use_dhcp: true,
            applied_resolver: "1.1.1.1".to_string(),
            applied_at: Utc::now(),
        };
        crate::assist::save_backup(&dir, &backup).unwrap();
        let timeline = build_config_timeline(&dir);
        assert!(timeline.iter().any(|line| line.contains("DNS Assist")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
