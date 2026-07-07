use crate::types::*;
use std::time::{Duration, Instant};

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_DOMAINS_PER_CHECK: usize = 4;

pub async fn probe_site_reachability(
    domains: &[String],
    environment: &EnvironmentSnapshot,
) -> SiteReachabilityStatus {
    let targets: Vec<String> = domains
        .iter()
        .take(MAX_DOMAINS_PER_CHECK)
        .map(|domain| domain.trim().to_ascii_lowercase())
        .filter(|domain| !domain.is_empty())
        .collect();

    if targets.is_empty() {
        return SiteReachabilityStatus {
            checked_domains: 0,
            success_count: 0,
            failure_count: 0,
            results: Vec::new(),
            summary: "No verification domains configured.".to_string(),
        };
    }

    let client = match build_http_client(environment) {
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
                    })
                    .collect(),
                summary: format!("Site reachability probe unavailable: {error}"),
            };
        }
    };

    let mut results = Vec::new();
    for domain in targets {
        let url = format!("https://{domain}/");
        let started = Instant::now();
        let response = client.head(&url).send().await;
        let elapsed = started.elapsed().as_millis() as f64;

        match response {
            Ok(response) => {
                let status = response.status().as_u16();
                let success = status < 500;
                results.push(SiteReachResult {
                    domain: domain.clone(),
                    success,
                    status_code: Some(status),
                    latency_ms: Some(elapsed),
                    error: if success {
                        None
                    } else {
                        Some(format!("HTTP {status}"))
                    },
                });
            }
            Err(error) => results.push(SiteReachResult {
                domain,
                success: false,
                status_code: None,
                latency_ms: Some(elapsed),
                error: Some(error.to_string()),
            }),
        }
    }

    summarize_site_reachability(results)
}

fn build_http_client(environment: &EnvironmentSnapshot) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(2));

    if environment.proxy.enabled {
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

pub fn summarize_site_reachability(results: Vec<SiteReachResult>) -> SiteReachabilityStatus {
    let checked_domains = results.len() as u8;
    let success_count = results.iter().filter(|result| result.success).count() as u8;
    let failure_count = checked_domains.saturating_sub(success_count);

    let summary = if checked_domains == 0 {
        "No sites checked.".to_string()
    } else if failure_count == 0 {
        format!("All {checked_domains} verification sites responded.")
    } else if success_count == 0 {
        "Verification sites are unreachable on the current path.".to_string()
    } else {
        format!(
            "{failure_count} of {checked_domains} verification sites failed over HTTPS."
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
}
