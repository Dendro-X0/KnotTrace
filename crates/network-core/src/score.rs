use crate::types::*;

pub fn score_health(probe: &ProbeResult) -> HealthScore {
    let mut reasons = Vec::new();
    let mut penalty = 0u8;

    if let Some(internet) = &probe.internet {
        if internet.loss_pct >= 50.0 {
            penalty += 50;
            reasons.push(format!(
                "Internet path unstable ({:.0}% connection failures)",
                internet.loss_pct
            ));
        } else if internet.loss_pct >= 15.0 {
            penalty += 25;
            reasons.push(format!(
                "Some connection loss to internet ({:.0}%)",
                internet.loss_pct
            ));
        }

        if internet.avg_ms >= 200.0 {
            penalty += 35;
            reasons.push(format!("High internet latency ({:.0} ms)", internet.avg_ms));
        } else if internet.avg_ms >= 100.0 {
            penalty += 20;
            reasons.push(format!("Elevated internet latency ({:.0} ms)", internet.avg_ms));
        } else if internet.avg_ms >= 60.0 {
            penalty += 10;
            reasons.push(format!("Moderate internet latency ({:.0} ms)", internet.avg_ms));
        }
    } else {
        penalty += 60;
        reasons.push("Unable to reach the internet".to_string());
    }

    if let Some(gateway) = &probe.gateway {
        if gateway.loss_pct >= 50.0 {
            penalty += 20;
            reasons.push("Local gateway is not responding reliably".to_string());
        } else if gateway.avg_ms >= 20.0 {
            penalty += 10;
            reasons.push(format!("Local gateway latency is high ({:.0} ms)", gateway.avg_ms));
        }
    }

    let dns_failures = probe.dns.iter().filter(|sample| !sample.success).count();
    if dns_failures > 0 {
        penalty += 25;
        reasons.push(format!("{dns_failures} DNS resolver(s) failed"));
    }

    if let Some(slowest_dns) = probe
        .dns
        .iter()
        .filter(|sample| sample.success)
        .max_by(|a, b| {
            a.latency_ms
                .partial_cmp(&b.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        if slowest_dns.latency_ms >= 150.0 {
            penalty += 25;
            reasons.push(format!(
                "DNS is slow ({:.0} ms via {})",
                slowest_dns.latency_ms, slowest_dns.resolver
            ));
        } else if slowest_dns.latency_ms >= 80.0 {
            penalty += 15;
            reasons.push(format!(
                "DNS latency is elevated ({:.0} ms)",
                slowest_dns.latency_ms
            ));
        }
    }

    let score = 100u8.saturating_sub(penalty.min(95));
    let grade = if score >= 80 {
        HealthGrade::Good
    } else if score >= 55 {
        HealthGrade::Fair
    } else {
        HealthGrade::Poor
    };

    let summary = match grade {
        HealthGrade::Good => "Connection looks healthy".to_string(),
        HealthGrade::Fair => "Connection is usable but could be smoother".to_string(),
        HealthGrade::Poor => "Connection quality is degraded".to_string(),
    };

    if reasons.is_empty() {
        reasons.push("Latency and DNS look normal".to_string());
    }

    HealthScore {
        grade,
        score,
        summary,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DnsProbe, LatencySample, ProbeResult};

    #[test]
    fn scores_poor_when_internet_unreachable() {
        let probe = ProbeResult {
            gateway: None,
            internet: None,
            dns: vec![DnsProbe {
                resolver: "1.1.1.1".to_string(),
                query: "example.com".to_string(),
                latency_ms: 20.0,
                success: true,
            }],
            duration_ms: 10,
        };

        let score = score_health(&probe);
        assert_eq!(score.grade, HealthGrade::Poor);
    }

    #[test]
    fn scores_good_for_low_latency_path() {
        let probe = ProbeResult {
            gateway: Some(LatencySample {
                target: "gateway".to_string(),
                avg_ms: 2.0,
                loss_pct: 0.0,
            }),
            internet: Some(LatencySample {
                target: "internet".to_string(),
                avg_ms: 25.0,
                loss_pct: 0.0,
            }),
            dns: vec![DnsProbe {
                resolver: "1.1.1.1".to_string(),
                query: "example.com".to_string(),
                latency_ms: 18.0,
                success: true,
            }],
            duration_ms: 100,
        };

        let score = score_health(&probe);
        assert_eq!(score.grade, HealthGrade::Good);
    }
}
