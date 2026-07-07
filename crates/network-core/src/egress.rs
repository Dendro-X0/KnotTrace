use crate::types::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

struct EgressEndpoint {
    provider: &'static str,
    url: &'static str,
    parser: fn(&str) -> Option<String>,
}

const ENDPOINTS: &[EgressEndpoint] = &[
    EgressEndpoint {
        provider: "cloudflare",
        url: "https://cloudflare.com/cdn-cgi/trace",
        parser: parse_cloudflare_trace_ip,
    },
    EgressEndpoint {
        provider: "ipify",
        url: "https://api.ipify.org?format=json",
        parser: parse_ipify_json,
    },
    EgressEndpoint {
        provider: "ifconfig",
        url: "https://ifconfig.me/ip",
        parser: parse_plain_ip,
    },
];

pub async fn probe_egress(environment: &EnvironmentSnapshot) -> EgressReport {
    let system_path = probe_path(EgressPathKind::System, environment, None).await;
    let tor_path = if environment.tor.socks_reachable {
        if let Some(endpoint) = environment.tor.socks_endpoint.as_deref() {
            Some(probe_path(EgressPathKind::TorSocks, environment, Some(endpoint)).await)
        } else {
            None
        }
    } else {
        None
    };

    let (primary_ip, confidence) = summarize_primary_ip(&system_path.endpoints);
    let summary = build_summary(primary_ip.as_deref(), confidence, tor_path.as_ref());

    EgressReport {
        primary_ip,
        confidence,
        system_path,
        tor_path,
        summary,
    }
}

async fn probe_path(
    kind: EgressPathKind,
    environment: &EnvironmentSnapshot,
    tor_socks: Option<&str>,
) -> EgressPathReport {
    let client = match build_client(environment, tor_socks) {
        Ok(client) => client,
        Err(error) => {
            return EgressPathReport {
                kind,
                primary_ip: None,
                endpoints: Vec::new(),
                summary: format!("Egress probe unavailable: {error}"),
            };
        }
    };

    let mut endpoints = Vec::new();
    for endpoint in ENDPOINTS {
        let started = Instant::now();
        let response = client.get(endpoint.url).send().await;
        let elapsed = started.elapsed().as_millis() as f64;

        match response {
            Ok(response) => {
                if !response.status().is_success() {
                    endpoints.push(EgressEndpointResult {
                        provider: endpoint.provider.to_string(),
                        ip: None,
                        latency_ms: Some(elapsed),
                        success: false,
                        error: Some(format!("HTTP {}", response.status())),
                    });
                    continue;
                }

                match response.text().await {
                    Ok(body) => {
                        let ip = (endpoint.parser)(&body);
                        endpoints.push(EgressEndpointResult {
                            provider: endpoint.provider.to_string(),
                            ip: ip.clone(),
                            latency_ms: Some(elapsed),
                            success: ip.is_some(),
                            error: if ip.is_some() {
                                None
                            } else {
                                Some("Could not parse IP from response".to_string())
                            },
                        });
                    }
                    Err(error) => endpoints.push(EgressEndpointResult {
                        provider: endpoint.provider.to_string(),
                        ip: None,
                        latency_ms: Some(elapsed),
                        success: false,
                        error: Some(error.to_string()),
                    }),
                }
            }
            Err(error) => endpoints.push(EgressEndpointResult {
                provider: endpoint.provider.to_string(),
                ip: None,
                latency_ms: Some(elapsed),
                success: false,
                error: Some(error.to_string()),
            }),
        }
    }

    let (primary_ip, confidence) = summarize_primary_ip(&endpoints);
    let summary = match (&primary_ip, confidence) {
        (Some(ip), EgressConfidence::High) => format!("Public egress IP is {ip} (confirmed)."),
        (Some(ip), _) => format!("Public egress IP appears to be {ip}."),
        (None, _) => "Could not determine public egress IP.".to_string(),
    };

    EgressPathReport {
        kind,
        primary_ip,
        endpoints,
        summary,
    }
}

fn build_client(
    environment: &EnvironmentSnapshot,
    tor_socks: Option<&str>,
) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(2));

    if let Some(socks) = tor_socks {
        let proxy_url = normalize_socks_url(socks);
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|error| format!("invalid Tor SOCKS URL: {error}"))?;
        builder = builder.proxy(proxy);
    } else if environment.proxy.enabled {
        if let Some(proxy_server) = environment.proxy.server.as_deref() {
            let proxy_url = normalize_proxy_url(proxy_server);
            let proxy = reqwest::Proxy::all(&proxy_url)
                .map_err(|error| format!("invalid proxy URL: {error}"))?;
            builder = builder.proxy(proxy);
        }
    }

    builder
        .build()
        .map_err(|error| format!("http client error: {error}"))
}

fn normalize_proxy_url(proxy_server: &str) -> String {
    if proxy_server.contains("://") {
        proxy_server.to_string()
    } else {
        format!("http://{proxy_server}")
    }
}

fn normalize_socks_url(socks_server: &str) -> String {
    if socks_server.contains("://") {
        socks_server.to_string()
    } else {
        format!("socks5h://{socks_server}")
    }
}

fn parse_cloudflare_trace_ip(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.strip_prefix("ip=").map(str::trim).map(str::to_string))
}

fn parse_ipify_json(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .get("ip")
        .and_then(|ip| ip.as_str())
        .map(str::to_string)
}

fn parse_plain_ip(body: &str) -> Option<String> {
    let ip = body.trim();
    if ip.contains('.') || ip.contains(':') {
        Some(ip.to_string())
    } else {
        None
    }
}

pub fn summarize_primary_ip(endpoints: &[EgressEndpointResult]) -> (Option<String>, EgressConfidence) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for endpoint in endpoints.iter().filter(|endpoint| endpoint.success) {
        if let Some(ip) = &endpoint.ip {
            *counts.entry(ip.clone()).or_default() += 1;
        }
    }

    if counts.is_empty() {
        return (None, EgressConfidence::Unknown);
    }

    let max_count = counts.values().copied().max().unwrap_or(0);
    let winners: Vec<String> = counts
        .into_iter()
        .filter(|(_, count)| *count == max_count)
        .map(|(ip, _)| ip)
        .collect();

    let success_count = endpoints.iter().filter(|endpoint| endpoint.success).count();
    let confidence = if winners.len() == 1 && max_count == success_count && success_count >= 2 {
        EgressConfidence::High
    } else if winners.len() == 1 && max_count >= 2 {
        EgressConfidence::Medium
    } else if winners.len() == 1 {
        EgressConfidence::Low
    } else {
        EgressConfidence::Low
    };

    (winners.first().cloned(), confidence)
}

pub fn egress_unstable(report: &EgressReport) -> bool {
    matches!(report.confidence, EgressConfidence::Low | EgressConfidence::Unknown)
        || report.primary_ip.is_none()
}

fn build_summary(
    primary_ip: Option<&str>,
    confidence: EgressConfidence,
    tor_path: Option<&EgressPathReport>,
) -> String {
    let base = match (primary_ip, confidence) {
        (Some(ip), EgressConfidence::High) => format!("Public IP {ip} confirmed across providers."),
        (Some(ip), EgressConfidence::Medium) => {
            format!("Public IP {ip} reported by multiple providers.")
        }
        (Some(ip), EgressConfidence::Low) => {
            format!("Public IP {ip} detected with low confidence.")
        }
        (Some(ip), EgressConfidence::Unknown) => format!("Public IP {ip} reported once."),
        (None, _) => "Public egress IP could not be determined.".to_string(),
    };

    if let Some(tor) = tor_path {
        if let (Some(system_ip), Some(tor_ip)) = (primary_ip, tor.primary_ip.as_deref()) {
            if system_ip != tor_ip {
                return format!("{base} Tor SOCKS egress differs ({tor_ip}).");
            }
        }
    }

    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_confidence_when_all_agree() {
        let endpoints = vec![
            EgressEndpointResult {
                provider: "a".to_string(),
                ip: Some("203.0.113.10".to_string()),
                latency_ms: Some(40.0),
                success: true,
                error: None,
            },
            EgressEndpointResult {
                provider: "b".to_string(),
                ip: Some("203.0.113.10".to_string()),
                latency_ms: Some(42.0),
                success: true,
                error: None,
            },
        ];

        let (ip, confidence) = summarize_primary_ip(&endpoints);
        assert_eq!(ip.as_deref(), Some("203.0.113.10"));
        assert_eq!(confidence, EgressConfidence::High);
    }

    #[test]
    fn parses_cloudflare_trace() {
        let body = "fl=1024f0\nh=cloudflare.com\nip=198.51.100.4\n";
        assert_eq!(
            parse_cloudflare_trace_ip(body).as_deref(),
            Some("198.51.100.4")
        );
    }
}
