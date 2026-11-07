use crate::types::{HealthReport, HistoryTrendPoint};
use std::path::Path;
use thiserror::Error;

const MAX_REPORTS: usize = 200;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct HistoryStore {
    conn: rusqlite::Connection,
}

impl HistoryStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS health_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                grade TEXT NOT NULL,
                score INTEGER NOT NULL,
                summary TEXT NOT NULL,
                report_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_health_reports_timestamp
            ON health_reports(timestamp DESC);
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn insert_report(&self, report: &HealthReport) -> Result<(), StoreError> {
        let payload = serde_json::to_string(report)?;
        self.conn.execute(
            "INSERT INTO health_reports (timestamp, grade, score, summary, report_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                report.timestamp.to_rfc3339(),
                format!("{:?}", report.score.grade).to_lowercase(),
                report.score.score,
                report.score.summary,
                payload,
            ],
        )?;
        self.prune_old_reports()?;
        Ok(())
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<HealthReport>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT report_json FROM health_reports
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let rows = statement.query_map([limit as i64], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut reports = Vec::new();
        for row in rows {
            reports.push(serde_json::from_str(&row?)?);
        }

        Ok(reports)
    }

    pub fn trend_points(&self, limit: usize) -> Result<Vec<HistoryTrendPoint>, StoreError> {
        let mut reports = self.recent(limit)?;
        reports.reverse();
        Ok(reports.into_iter().map(report_to_trend_point).collect())
    }

    fn prune_old_reports(&self) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM health_reports
             WHERE id NOT IN (
               SELECT id FROM health_reports
               ORDER BY timestamp DESC
               LIMIT ?1
             )",
            [MAX_REPORTS as i64],
        )?;
        Ok(())
    }
}

fn report_to_trend_point(report: HealthReport) -> HistoryTrendPoint {
    let integrity = report.dns_integrity.as_ref();
    let path = report.proxy_path_report.as_ref();
    HistoryTrendPoint {
        timestamp: report.timestamp,
        score: report.score.score,
        grade: report.score.grade,
        dns_latency_ms: report
            .probe
            .dns
            .iter()
            .filter(|sample| sample.success)
            .map(|sample| sample.latency_ms)
            .min_by(|left, right| {
                left.partial_cmp(right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        internet_latency_ms: report.probe.internet.as_ref().map(|sample| sample.avg_ms),
        gateway_latency_ms: report.probe.gateway.as_ref().map(|sample| sample.avg_ms),
        dns_integrity_state: integrity.map(|status| status.state),
        dns_integrity_confidence: integrity.map(|status| status.confidence),
        dns_integrity_mismatch_count: integrity.map(|status| status.mismatch_count),
        slowdown_shape: report.diagnosis.as_ref().map(|diagnosis| diagnosis.slowdown_shape),
        proxy_enabled: Some(report.environment.proxy.enabled),
        proxy_only_failure_count: path.map(|report| report.proxy_only_failure_count),
        likely_provider_side: path.map(|report| report.likely_provider_side),
        egress_ip: report
            .egress
            .as_ref()
            .and_then(|egress| egress.primary_ip.clone()),
        upstream_claim: report
            .upstream_pool
            .as_ref()
            .map(|proof| proof.claim),
        active_path: report.path_break.as_ref().map(|pb| pb.active_path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn bare_report(active: Option<PathBreakPath>) -> HealthReport {
        HealthReport {
            timestamp: Utc::now(),
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
                grade: HealthGrade::Fair,
                score: 70,
                summary: "fair".into(),
                reasons: vec![],
            },
            dns_integrity: None,
            diagnosis: Some(NetworkDiagnosis {
                summary: "partial".into(),
                primary_bottleneck: None,
                slowdown_shape: SlowdownShape::PartialSiteFailure,
                confidence: DiagnosisConfidence::Medium,
                hints: vec![],
            }),
            stability: None,
            site_reachability: None,
            egress: Some(EgressReport {
                primary_ip: Some("203.0.113.9".into()),
                confidence: EgressConfidence::High,
                system_path: EgressPathReport {
                    kind: EgressPathKind::System,
                    primary_ip: Some("203.0.113.9".into()),
                    endpoints: vec![],
                    summary: "system".into(),
                },
                tor_path: None,
                summary: "egress".into(),
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
            path_break: active.map(|active_path| PathBreakReport {
                local_ok: true,
                local_issues: vec![],
                active_path,
                paths_present: vec![active_path],
                break_hop: PathBreakHop::None,
                environment_class: PathEnvironmentClass::HomeLan,
                confidence: PathBreakConfidence::Medium,
                title: "path".into(),
                summary: "path".into(),
                evidence: vec![],
                not_checked: vec![],
            }),
            integrity: None,
            egress_timeline: None,
        }
    }

    #[test]
    fn trend_point_includes_active_path_and_egress() {
        let point = report_to_trend_point(bare_report(Some(PathBreakPath::SystemProxy)));
        assert_eq!(point.active_path, Some(PathBreakPath::SystemProxy));
        assert_eq!(point.egress_ip.as_deref(), Some("203.0.113.9"));
        assert_eq!(point.grade, HealthGrade::Fair);
        assert_eq!(point.slowdown_shape, Some(SlowdownShape::PartialSiteFailure));
    }

    #[test]
    fn trend_point_omits_active_path_when_path_break_missing() {
        let point = report_to_trend_point(bare_report(None));
        assert_eq!(point.active_path, None);
    }
}
