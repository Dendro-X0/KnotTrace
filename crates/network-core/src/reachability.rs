use crate::types::*;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

pub const PROXY_VERIFICATION_DOMAINS: &[&str] = &[
    "www.google.com",
    "www.cloudflare.com",
    "github.com",
    "www.microsoft.com",
];

#[derive(Debug, Clone, Copy)]
pub struct ReachabilityProbeOptions {
    pub timeout: Duration,
    pub max_domains_per_check: usize,
}

impl Default for ReachabilityProbeOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(4),
            max_domains_per_check: 4,
        }
    }
}

pub async fn probe_site_reachability(
    domains: &[String],
    environment: &EnvironmentSnapshot,
) -> SiteReachabilityStatus {
    probe_site_reachability_with_options(domains, environment, ReachabilityProbeOptions::default())
        .await
}

pub async fn probe_site_reachability_with_options(
    domains: &[String],
    environment: &EnvironmentSnapshot,
    options: ReachabilityProbeOptions,
) -> SiteReachabilityStatus {
    let targets = build_reachability_targets(domains, environment, options.max_domains_per_check);

    if targets.is_empty() {
        return SiteReachabilityStatus {
            checked_domains: 0,
            success_count: 0,
            failure_count: 0,
            results: Vec::new(),
            summary: "No verification domains configured.".to_string(),
        };
    }

    let use_proxy = environment.proxy.enabled;
    let client = match build_http_client(environment, options.timeout, use_proxy) {
        Ok(client) => client,
        Err(error) => {
            return SiteReachabilityStatus {
                checked_domains: targets.len() as u8,
                success_count: 0,
                failure_count: targets.len() as u8,
                results: targets
                    .into_iter()
                    .map(|domain| SiteReachResult {
                        domain,
                        success: false,
                        status_code: None,
                        latency_ms: None,
                        error: Some(error.clone()),
                        error_kind: Some(SiteReachErrorKind::Unknown),
                    })
                    .collect(),
                summary: format!("Site reachability probe unavailable: {error}"),
            };
        }
    };

    let results = probe_targets(&targets, client).await;
    summarize_site_reachability(results)
}

pub async fn probe_proxy_path_report(
    domains: &[String],
    environment: &EnvironmentSnapshot,
    options: ReachabilityProbeOptions,
) -> Option<ProxyPathReport> {
    if !environment.proxy.enabled {
        return None;
    }

    let targets = build_reachability_targets(domains, environment, options.max_domains_per_check);
    if targets.is_empty() {
        return None;
    }

    let proxy_client = match build_http_client(environment, options.timeout, true) {
        Ok(client) => client,
        Err(error) => {
            return Some(empty_proxy_path_report(&targets, &error));
        }
    };
    let direct_client = match build_http_client(environment, options.timeout, false) {
        Ok(client) => client,
        Err(error) => {
            return Some(empty_proxy_path_report(&targets, &error));
        }
    };

    let proxy_task = probe_targets(&targets, proxy_client);
    let direct_task = probe_targets(&targets, direct_client);
    let (proxy_results, direct_results) = tokio::join!(proxy_task, direct_task);

    Some(build_proxy_path_report(targets, proxy_results, direct_results))
}

fn empty_proxy_path_report(targets: &[String], error: &str) -> ProxyPathReport {
    let failed = targets
        .iter()
        .map(|domain| SiteReachResult {
            domain: domain.clone(),
            success: false,
            status_code: None,
            latency_ms: None,
            error: Some(error.to_string()),
            error_kind: Some(SiteReachErrorKind::Unknown),
        })
        .collect::<Vec<_>>();

    build_proxy_path_report(
        targets.to_vec(),
        failed.clone(),
        failed,
    )
}

async fn probe_targets(targets: &[String], client: reqwest::Client) -> Vec<SiteReachResult> {
    let mut join_set = JoinSet::new();
    for (index, domain) in targets.iter().cloned().enumerate() {
        let client = client.clone();
        join_set.spawn(async move {
            let result = probe_domain(&domain, &client).await;
            (index, result)
        });
    }

    let mut ordered = Vec::new();
    while let Some(joined) = join_set.join_next().await {
        if let Ok((index, result)) = joined {
            ordered.push((index, result));
        }
    }
    ordered.sort_by_key(|(index, _)| *index);
    ordered.into_iter().map(|(_, result)| result).collect()
}

async fn probe_domain(domain: &str, client: &reqwest::Client) -> SiteReachResult {
    let url = format!("https://{domain}/");
    let started = Instant::now();
    let response = client.head(&url).send().await;
    let elapsed = started.elapsed().as_millis() as f64;

    match response {
        Ok(response) => {
            let status = response.status().as_u16();
            let success = status < 500;
            let error_kind = if success {
                if status >= 400 {
                    Some(SiteReachErrorKind::HttpBlocked)
                } else {
                    None
                }
            } else {
                Some(SiteReachErrorKind::HttpServer)
            };
            SiteReachResult {
                domain: domain.to_string(),
                success,
                status_code: Some(status),
                latency_ms: Some(elapsed),
                error: if success {
                    if status >= 400 {
                        Some(format!("HTTP {status}"))
                    } else {
                        None
                    }
                } else {
                    Some(format!("HTTP {status}"))
                },
                error_kind,
            }
        }
        Err(error) => {
            let error_kind = classify_reachability_error(&error, None);
            SiteReachResult {
                domain: domain.to_string(),
                success: false,
                status_code: None,
                latency_ms: Some(elapsed),
                error: Some(error.to_string()),
                error_kind: Some(error_kind),
            }
        }
    }
}

pub fn classify_reachability_error(error: &reqwest::Error, status: Option<u16>) -> SiteReachErrorKind {
    if let Some(status) = status {
        if status >= 500 {
            return SiteReachErrorKind::HttpServer;
        }
        if status == 403 || status == 407 || status == 502 || status == 503 {
            return SiteReachErrorKind::Proxy;
        }
        if status >= 400 {
            return SiteReachErrorKind::HttpBlocked;
        }
    }

    if error.is_timeout() {
        return SiteReachErrorKind::Timeout;
    }

    let message = error.to_string().to_ascii_lowercase();
    if message.contains("certificate")
        || message.contains("tls")
        || message.contains("ssl")
        || message.contains("handshake")
    {
        return SiteReachErrorKind::Tls;
    }
    if message.contains("dns") || message.contains("resolve") || message.contains("lookup") {
        return SiteReachErrorKind::Dns;
    }
    if message.contains("reset") || message.contains("broken pipe") || message.contains("aborted") {
        return SiteReachErrorKind::ConnectionReset;
    }
    if message.contains("proxy") || message.contains("407") {
        return SiteReachErrorKind::Proxy;
    }

    SiteReachErrorKind::Unknown
}

fn build_proxy_path_report(
    targets: Vec<String>,
    proxy_results: Vec<SiteReachResult>,
    direct_results: Vec<SiteReachResult>,
) -> ProxyPathReport {
    let comparisons: Vec<ProxyPathDomainComparison> = targets
        .into_iter()
        .zip(proxy_results.into_iter().zip(direct_results))
        .map(|(domain, (proxy, direct))| {
            let proxy_only_failure = !proxy.success && direct.success;
            ProxyPathDomainComparison {
                domain,
                proxy,
                direct,
                proxy_only_failure,
            }
        })
        .collect();

    let checked_domains = comparisons.len() as u8;
    let proxy_failure_count = comparisons.iter().filter(|row| !row.proxy.success).count() as u8;
    let direct_failure_count = comparisons.iter().filter(|row| !row.direct.success).count() as u8;
    let proxy_only_failure_count = comparisons
        .iter()
        .filter(|row| row.proxy_only_failure)
        .count() as u8;

    let likely_provider_side = proxy_only_failure_count > 0
        || (proxy_failure_count >= 2 && direct_failure_count == 0);

    let confidence = if proxy_only_failure_count >= 2 {
        ProxyPathConfidence::High
    } else if proxy_only_failure_count == 1 || (proxy_failure_count >= 2 && direct_failure_count == 0)
    {
        ProxyPathConfidence::Medium
    } else if proxy_failure_count > 0 {
        ProxyPathConfidence::Low
    } else {
        ProxyPathConfidence::Low
    };

    let summary = summarize_proxy_path_report(
        checked_domains,
        proxy_failure_count,
        direct_failure_count,
        proxy_only_failure_count,
        likely_provider_side,
        &comparisons,
    );

    ProxyPathReport {
        comparisons,
        checked_domains,
        proxy_failure_count,
        direct_failure_count,
        proxy_only_failure_count,
        confidence,
        likely_provider_side,
        summary,
    }
}

fn summarize_proxy_path_report(
    checked: u8,
    proxy_failures: u8,
    direct_failures: u8,
    proxy_only_failures: u8,
    likely_provider_side: bool,
    comparisons: &[ProxyPathDomainComparison],
) -> String {
    if checked == 0 {
        return "No proxy path targets checked.".to_string();
    }

    if proxy_only_failures > 0 {
        let sample = comparisons
            .iter()
            .filter(|row| row.proxy_only_failure)
            .take(2)
            .map(|row| row.domain.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return format!(
            "{proxy_only_failures} of {checked} major sites failed only on the proxy path (example: {sample}). This strongly suggests upstream proxy or VPN quality, not your local network."
        );
    }

    if likely_provider_side && proxy_failures >= 2 && direct_failures == 0 {
        return format!(
            "{proxy_failures} of {checked} sites failed through the proxy while direct access succeeded. The active proxy path is the likely bottleneck."
        );
    }

    if proxy_failures == 0 && direct_failures == 0 {
        return format!("All {checked} proxy verification sites responded on both proxy and direct paths.");
    }

    if proxy_failures > 0 && direct_failures > 0 {
        return format!(
            "{proxy_failures} proxy failures and {direct_failures} direct failures across {checked} sites. Both paths may be impaired."
        );
    }

    format!(
        "{proxy_failures} of {checked} sites failed over the proxy path. Review per-domain errors for filtering, resets, or timeouts."
    )
}

pub fn proxy_verification_failures(reachability: &SiteReachabilityStatus) -> Vec<&'static str> {
    let mut failures = Vec::new();
    for domain in PROXY_VERIFICATION_DOMAINS {
        if reachability
            .results
            .iter()
            .any(|result| !result.success && result.domain.eq_ignore_ascii_case(domain))
        {
            failures.push(*domain);
        }
    }
    failures
}

fn build_http_client(
    environment: &EnvironmentSnapshot,
    timeout: Duration,
    use_system_proxy: bool,
) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(2));

    if use_system_proxy && environment.proxy.enabled {
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

fn build_reachability_targets(
    domains: &[String],
    environment: &EnvironmentSnapshot,
    max: usize,
) -> Vec<String> {
    if max == 0 {
        return Vec::new();
    }

    let mut targets: Vec<String> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    if environment.proxy.enabled {
        for domain in PROXY_VERIFICATION_DOMAINS {
            let normalized = domain.trim().to_ascii_lowercase();
            if normalized.is_empty() || !seen.insert(normalized.clone()) {
                continue;
            }
            targets.push(normalized);
            if targets.len() >= max {
                return targets;
            }
        }
    }

    for domain in domains.iter() {
        let normalized = domain.trim().to_ascii_lowercase();
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        targets.push(normalized);
        if targets.len() >= max {
            break;
        }
    }

    targets
}

pub fn summarize_site_reachability(results: Vec<SiteReachResult>) -> SiteReachabilityStatus {
    let checked_domains = results.len() as u8;
    let success_count = results.iter().filter(|result| result.success).count() as u8;
    let failure_count = checked_domains.saturating_sub(success_count);

    let failed_domains: Vec<&str> = results
        .iter()
        .filter(|result| !result.success)
        .map(|result| result.domain.as_str())
        .collect();

    let summary = if checked_domains == 0 {
        "No sites checked.".to_string()
    } else if failure_count == 0 {
        format!("All {checked_domains} verification sites responded.")
    } else if success_count == 0 {
        "Verification sites are unreachable on the current path.".to_string()
    } else {
        let sample = failed_domains
            .first()
            .copied()
            .unwrap_or("unknown domain");
        format!(
            "{failure_count} of {checked_domains} verification sites failed over HTTPS (example: {sample})."
        )
    };

    SiteReachabilityStatus {
        checked_domains,
        success_count,
        failure_count,
        results,
        summary,
    }
}

pub fn site_access_degraded(status: &SiteReachabilityStatus) -> bool {
    status.failure_count > 0 && status.failure_count * 2 >= status.checked_domains
}

pub fn error_kind_label(kind: SiteReachErrorKind) -> &'static str {
    match kind {
        SiteReachErrorKind::Timeout => "timeout",
        SiteReachErrorKind::ConnectionReset => "connection reset",
        SiteReachErrorKind::Tls => "TLS error",
        SiteReachErrorKind::HttpServer => "HTTP server error",
        SiteReachErrorKind::HttpBlocked => "HTTP blocked",
        SiteReachErrorKind::Proxy => "proxy error",
        SiteReachErrorKind::Dns => "DNS error",
        SiteReachErrorKind::Unknown => "unknown error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_majority_failures_as_degraded() {
        let status = SiteReachabilityStatus {
            checked_domains: 4,
            success_count: 1,
            failure_count: 3,
            results: Vec::new(),
            summary: String::new(),
        };
        assert!(site_access_degraded(&status));
    }

    #[test]
    fn ignores_single_failure() {
        let status = SiteReachabilityStatus {
            checked_domains: 4,
            success_count: 3,
            failure_count: 1,
            results: Vec::new(),
            summary: String::new(),
        };
        assert!(!site_access_degraded(&status));
    }

    #[test]
    fn proxy_targets_are_prepended_and_deduped() {
        let mut env = EnvironmentSnapshot {
            hostname: "host".to_string(),
            interfaces: Vec::new(),
            dns_servers: Vec::new(),
            proxy: ProxySettings {
                enabled: true,
                server: Some("127.0.0.1:7890".to_string()),
                source: "test".to_string(),
            },
            tags: Vec::new(),
            default_gateway: None,
            active_interface: None,
            tor: default_tor_status(),
        };

        let domains = vec![
            "github.com".to_string(),
            "example.com".to_string(),
        ];

        let targets = build_reachability_targets(&domains, &env, 3);
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0], "www.google.com");
        assert_eq!(targets[1], "www.cloudflare.com");
        assert_eq!(targets[2], "github.com");

        env.proxy.enabled = false;
        let targets_no_proxy = build_reachability_targets(&domains, &env, 3);
        assert_eq!(targets_no_proxy, vec!["github.com".to_string(), "example.com".to_string()]);
    }

    #[test]
    fn proxy_only_failures_mark_provider_side() {
        let report = build_proxy_path_report(
            vec!["github.com".to_string(), "www.google.com".to_string()],
            vec![
                SiteReachResult {
                    domain: "github.com".to_string(),
                    success: false,
                    status_code: None,
                    latency_ms: Some(100.0),
                    error: Some("reset".to_string()),
                    error_kind: Some(SiteReachErrorKind::ConnectionReset),
                },
                SiteReachResult {
                    domain: "www.google.com".to_string(),
                    success: false,
                    status_code: None,
                    latency_ms: Some(120.0),
                    error: Some("timeout".to_string()),
                    error_kind: Some(SiteReachErrorKind::Timeout),
                },
            ],
            vec![
                SiteReachResult {
                    domain: "github.com".to_string(),
                    success: true,
                    status_code: Some(200),
                    latency_ms: Some(40.0),
                    error: None,
                    error_kind: None,
                },
                SiteReachResult {
                    domain: "www.google.com".to_string(),
                    success: true,
                    status_code: Some(200),
                    latency_ms: Some(35.0),
                    error: None,
                    error_kind: None,
                },
            ],
        );

        assert!(report.likely_provider_side);
        assert_eq!(report.proxy_only_failure_count, 2);
        assert_eq!(report.confidence, ProxyPathConfidence::High);
        assert!(report.summary.contains("proxy path"));
    }
}
