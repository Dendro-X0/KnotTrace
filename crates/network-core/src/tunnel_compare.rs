use crate::types::*;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

const COMPARE_DOMAINS: &[&str] = &[
    "www.cloudflare.com",
    "github.com",
    "www.microsoft.com",
];

const EGRESS_URL: &str = "https://cloudflare.com/cdn-cgi/trace";

#[derive(Debug, Clone, Copy)]
pub struct TunnelCompareOptions {
    pub timeout: Duration,
    pub max_domains: usize,
}

impl Default for TunnelCompareOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(4),
            max_domains: 3,
        }
    }
}

pub fn should_compare_tunnels(environment: &EnvironmentSnapshot) -> bool {
    environment.tor.detected
        || environment.proxy.enabled
        || environment.tags.contains(&EnvironmentTag::Vpn)
        || environment.tags.contains(&EnvironmentTag::Tor)
}

pub async fn probe_tunnel_path_compare(
    environment: &EnvironmentSnapshot,
    options: TunnelCompareOptions,
) -> Option<TunnelPathCompareReport> {
    if !should_compare_tunnels(environment) {
        return None;
    }

    let domains: Vec<String> = COMPARE_DOMAINS
        .iter()
        .take(options.max_domains.max(1))
        .map(|domain| (*domain).to_string())
        .collect();

    let vpn_detected = environment.tags.contains(&EnvironmentTag::Vpn);
    let tor_detected = environment.tor.detected || environment.tags.contains(&EnvironmentTag::Tor);
    let tor_socks_reachable = environment.tor.socks_reachable;
    let proxy_enabled = environment.proxy.enabled;

    let direct_task = probe_path_sample(
        TunnelPathKind::Direct,
        "Direct",
        environment,
        PathMode::Direct,
        &domains,
        options.timeout,
        if vpn_detected {
            Some("VPN/TUN often captures all OS traffic, so Direct may still traverse the tunnel.".to_string())
        } else {
            None
        },
    );

    let system_task = if proxy_enabled {
        Some(probe_path_sample(
            TunnelPathKind::SystemProxy,
            "System proxy",
            environment,
            PathMode::SystemProxy,
            &domains,
            options.timeout,
            None,
        ))
    } else if vpn_detected {
        Some(probe_path_sample(
            TunnelPathKind::SystemProxy,
            "System (VPN)",
            environment,
            PathMode::Direct,
            &domains,
            options.timeout,
            Some("No HTTP proxy is set; this is the default OS path (likely through the VPN).".to_string()),
        ))
    } else {
        None
    };

    let mut tor_unavailable: Option<TunnelPathSample> = None;
    let tor_task = if tor_detected {
        if tor_socks_reachable {
            if let Some(endpoint) = environment.tor.socks_endpoint.clone() {
                Some(probe_path_sample(
                    TunnelPathKind::TorSocks,
                    "Tor SOCKS",
                    environment,
                    PathMode::TorSocks(endpoint),
                    &domains,
                    options.timeout,
                    Some("Tor trades speed for anonymity — higher latency is expected.".to_string()),
                ))
            } else {
                None
            }
        } else {
            tor_unavailable = Some(TunnelPathSample {
                kind: TunnelPathKind::TorSocks,
                label: "Tor SOCKS".to_string(),
                available: false,
                egress_ip: None,
                reachability: Vec::new(),
                median_latency_ms: None,
                success_count: 0,
                failure_count: 0,
                note: Some(
                    "Tor appears configured but SOCKS is unreachable. This is a Tor client/bootstrap issue, not a total internet outage."
                        .to_string(),
                ),
            });
            None
        }
    } else {
        None
    };

    let (direct, system, tor) = match (system_task, tor_task) {
        (Some(system), Some(tor)) => {
            let (direct, system, tor) = tokio::join!(direct_task, system, tor);
            (direct, Some(system), Some(tor))
        }
        (Some(system), None) => {
            let (direct, system) = tokio::join!(direct_task, system);
            (direct, Some(system), None)
        }
        (None, Some(tor)) => {
            let (direct, tor) = tokio::join!(direct_task, tor);
            (direct, None, Some(tor))
        }
        (None, None) => (direct_task.await, None, None),
    };

    let mut paths = vec![direct];
    if let Some(system) = system {
        paths.push(system);
    }
    if let Some(tor) = tor.or(tor_unavailable) {
        paths.push(tor);
    }

    Some(build_compare_report(
        paths,
        vpn_detected,
        tor_detected,
        tor_socks_reachable,
        proxy_enabled,
    ))
}

enum PathMode {
    Direct,
    SystemProxy,
    TorSocks(String),
}

async fn probe_path_sample(
    kind: TunnelPathKind,
    label: &str,
    environment: &EnvironmentSnapshot,
    mode: PathMode,
    domains: &[String],
    timeout: Duration,
    note: Option<String>,
) -> TunnelPathSample {
    let client = match build_client(environment, &mode, timeout) {
        Ok(client) => client,
        Err(error) => {
            return TunnelPathSample {
                kind,
                label: label.to_string(),
                available: false,
                egress_ip: None,
                reachability: Vec::new(),
                median_latency_ms: None,
                success_count: 0,
                failure_count: 0,
                note: Some(format!("{label} probe unavailable: {error}")),
            };
        }
    };

    let reachability = probe_domains(domains, &client).await;
    let success_count = reachability.iter().filter(|row| row.success).count() as u8;
    let failure_count = reachability.len().saturating_sub(success_count as usize) as u8;
    let median_latency_ms = median_latency(&reachability);
    let egress_ip = probe_egress_ip(&client).await;

    TunnelPathSample {
        kind,
        label: label.to_string(),
        available: true,
        egress_ip,
        reachability,
        median_latency_ms,
        success_count,
        failure_count,
        note,
    }
}

fn build_client(
    environment: &EnvironmentSnapshot,
    mode: &PathMode,
    timeout: Duration,
) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(2));

    match mode {
        PathMode::Direct => {
            builder = builder.no_proxy();
        }
        PathMode::SystemProxy => {
            if environment.proxy.enabled {
                if let Some(proxy_server) = environment.proxy.server.as_deref() {
                    let proxy_url = normalize_proxy_url(proxy_server);
                    let proxy = reqwest::Proxy::all(&proxy_url)
                        .map_err(|error| format!("invalid proxy URL: {error}"))?;
                    builder = builder.proxy(proxy);
                }
            }
        }
        PathMode::TorSocks(endpoint) => {
            let proxy_url = normalize_socks_url(endpoint);
            let proxy = reqwest::Proxy::all(&proxy_url)
                .map_err(|error| format!("invalid Tor SOCKS URL: {error}"))?;
            builder = builder.proxy(proxy);
        }
    }

    builder
        .build()
        .map_err(|error| format!("http client error: {error}"))
}

async fn probe_domains(domains: &[String], client: &reqwest::Client) -> Vec<SiteReachResult> {
    let mut join_set = JoinSet::new();
    for (index, domain) in domains.iter().cloned().enumerate() {
        let client = client.clone();
        join_set.spawn(async move {
            let url = format!("https://{domain}/");
            let started = Instant::now();
            let response = client.head(&url).send().await;
            let elapsed = started.elapsed().as_millis() as f64;
            let result = match response {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let success = status < 500;
                    SiteReachResult {
                        domain,
                        success,
                        status_code: Some(status),
                        latency_ms: Some(elapsed),
                        error: if success && status >= 400 {
                            Some(format!("HTTP {status}"))
                        } else if success {
                            None
                        } else {
                            Some(format!("HTTP {status}"))
                        },
                        error_kind: if success {
                            if status >= 400 {
                                Some(SiteReachErrorKind::HttpBlocked)
                            } else {
                                None
                            }
                        } else {
                            Some(SiteReachErrorKind::HttpServer)
                        },
                    }
                }
                Err(error) => SiteReachResult {
                    domain,
                    success: false,
                    status_code: None,
                    latency_ms: Some(elapsed),
                    error: Some(error.to_string()),
                    error_kind: Some(crate::reachability::classify_reachability_error(&error, None)),
                },
            };
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

async fn probe_egress_ip(client: &reqwest::Client) -> Option<String> {
    let response = client.get(EGRESS_URL).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.text().await.ok()?;
    body.lines()
        .find_map(|line| line.strip_prefix("ip=").map(str::trim).map(str::to_string))
}

fn median_latency(results: &[SiteReachResult]) -> Option<f64> {
    let mut values: Vec<f64> = results
        .iter()
        .filter(|row| row.success)
        .filter_map(|row| row.latency_ms)
        .collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        Some((values[mid - 1] + values[mid]) / 2.0)
    } else {
        Some(values[mid])
    }
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

pub fn build_compare_report(
    paths: Vec<TunnelPathSample>,
    vpn_detected: bool,
    tor_detected: bool,
    tor_socks_reachable: bool,
    proxy_enabled: bool,
) -> TunnelPathCompareReport {
    let tor_only_failures = tor_only_failure_domains(&paths);
    let expectation = build_expectation(
        vpn_detected,
        tor_detected,
        tor_socks_reachable,
        proxy_enabled,
        &paths,
    );
    let summary = build_summary(&paths, &tor_only_failures, tor_detected, tor_socks_reachable);

    TunnelPathCompareReport {
        paths,
        tor_only_failures,
        expectation,
        summary,
        vpn_detected,
        tor_detected,
        tor_socks_reachable,
        proxy_enabled,
    }
}

fn tor_only_failure_domains(paths: &[TunnelPathSample]) -> Vec<String> {
    let Some(tor) = paths.iter().find(|path| path.kind == TunnelPathKind::TorSocks) else {
        return Vec::new();
    };
    let Some(direct) = paths.iter().find(|path| path.kind == TunnelPathKind::Direct) else {
        return Vec::new();
    };

    tor.reachability
        .iter()
        .filter(|row| !row.success)
        .filter(|row| {
            direct
                .reachability
                .iter()
                .any(|direct_row| direct_row.domain == row.domain && direct_row.success)
        })
        .map(|row| row.domain.clone())
        .collect()
}

fn build_expectation(
    vpn_detected: bool,
    tor_detected: bool,
    tor_socks_reachable: bool,
    proxy_enabled: bool,
    paths: &[TunnelPathSample],
) -> String {
    let mut parts = Vec::new();

    if tor_detected {
        if tor_socks_reachable {
            parts.push(
                "Tor is for anonymity, not speed. Higher ping and lower Mbps than Direct are expected — KnotTrace will not try to accelerate Tor."
                    .to_string(),
            );
            if let (Some(direct), Some(tor)) = (
                paths.iter().find(|path| path.kind == TunnelPathKind::Direct),
                paths.iter().find(|path| path.kind == TunnelPathKind::TorSocks),
            ) {
                if let (Some(direct_ms), Some(tor_ms)) =
                    (direct.median_latency_ms, tor.median_latency_ms)
                {
                    if tor_ms > direct_ms {
                        let ratio = tor_ms / direct_ms.max(1.0);
                        parts.push(format!(
                            "Sampled median latency: Direct {direct_ms:.0} ms vs Tor {tor_ms:.0} ms (~{ratio:.1}×)."
                        ));
                    }
                }
            }
        } else {
            parts.push(
                "Tor SOCKS is not reachable. Fix or wait for the Tor client bootstrap — this is not evidence that your ISP link is down."
                    .to_string(),
            );
        }
    }

    if vpn_detected {
        parts.push(
            "A VPN/TUN interface is active. Extra latency and MTU sensitivity are normal; KnotTrace does not control vendor VPN apps."
                .to_string(),
        );
    }

    if proxy_enabled && !tor_detected {
        parts.push(
            "System proxy is enabled. Compare System proxy vs Direct above; provider-side failures belong upstream, not on your NIC."
                .to_string(),
        );
    }

    if parts.is_empty() {
        "No encrypted tunnel expectation notes for this path.".to_string()
    } else {
        parts.join(" ")
    }
}

fn build_summary(
    paths: &[TunnelPathSample],
    tor_only_failures: &[String],
    tor_detected: bool,
    tor_socks_reachable: bool,
) -> String {
    if tor_detected && !tor_socks_reachable {
        return "Tor is detected but SOCKS is down — treat this as a Tor client issue.".to_string();
    }

    let labels: Vec<String> = paths
        .iter()
        .filter(|path| path.available)
        .map(|path| {
            let latency = path
                .median_latency_ms
                .map(|ms| format!("{ms:.0} ms"))
                .unwrap_or_else(|| "n/a".to_string());
            format!(
                "{} {}/{} ok · {latency}",
                path.label, path.success_count, path.success_count + path.failure_count
            )
        })
        .collect();

    let base = if labels.is_empty() {
        "Tunnel path compare had no usable samples.".to_string()
    } else {
        format!("Path compare: {}.", labels.join("; "))
    };

    if tor_only_failures.is_empty() {
        base
    } else {
        format!(
            "{base} Tor-only failures: {}.",
            tor_only_failures.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_reach(domain: &str, success: bool, latency: f64) -> SiteReachResult {
        SiteReachResult {
            domain: domain.to_string(),
            success,
            status_code: if success { Some(200) } else { None },
            latency_ms: Some(latency),
            error: if success {
                None
            } else {
                Some("timeout".to_string())
            },
            error_kind: if success {
                None
            } else {
                Some(SiteReachErrorKind::Timeout)
            },
        }
    }

    #[test]
    fn expectation_states_tor_is_not_for_speed() {
        let report = build_compare_report(
            vec![
                TunnelPathSample {
                    kind: TunnelPathKind::Direct,
                    label: "Direct".to_string(),
                    available: true,
                    egress_ip: Some("1.2.3.4".to_string()),
                    reachability: vec![sample_reach("github.com", true, 40.0)],
                    median_latency_ms: Some(40.0),
                    success_count: 1,
                    failure_count: 0,
                    note: None,
                },
                TunnelPathSample {
                    kind: TunnelPathKind::TorSocks,
                    label: "Tor SOCKS".to_string(),
                    available: true,
                    egress_ip: Some("5.6.7.8".to_string()),
                    reachability: vec![sample_reach("github.com", true, 420.0)],
                    median_latency_ms: Some(420.0),
                    success_count: 1,
                    failure_count: 0,
                    note: None,
                },
            ],
            false,
            true,
            true,
            false,
        );

        assert!(report.expectation.to_ascii_lowercase().contains("anonymity"));
        assert!(report.expectation.to_ascii_lowercase().contains("not speed") || report.expectation.contains("accelerate") || report.expectation.contains("expected"));
        assert!(report.summary.contains("Path compare"));
    }

    #[test]
    fn finds_tor_only_failures() {
        let report = build_compare_report(
            vec![
                TunnelPathSample {
                    kind: TunnelPathKind::Direct,
                    label: "Direct".to_string(),
                    available: true,
                    egress_ip: None,
                    reachability: vec![sample_reach("github.com", true, 30.0)],
                    median_latency_ms: Some(30.0),
                    success_count: 1,
                    failure_count: 0,
                    note: None,
                },
                TunnelPathSample {
                    kind: TunnelPathKind::TorSocks,
                    label: "Tor SOCKS".to_string(),
                    available: true,
                    egress_ip: None,
                    reachability: vec![sample_reach("github.com", false, 900.0)],
                    median_latency_ms: None,
                    success_count: 0,
                    failure_count: 1,
                    note: None,
                },
            ],
            false,
            true,
            true,
            false,
        );

        assert_eq!(report.tor_only_failures, vec!["github.com".to_string()]);
    }

    #[test]
    fn socks_down_expectation_blames_tor_client() {
        let report = build_compare_report(
            vec![TunnelPathSample {
                kind: TunnelPathKind::TorSocks,
                label: "Tor SOCKS".to_string(),
                available: false,
                egress_ip: None,
                reachability: Vec::new(),
                median_latency_ms: None,
                success_count: 0,
                failure_count: 0,
                note: Some("unreachable".to_string()),
            }],
            false,
            true,
            false,
            false,
        );

        assert!(report.expectation.to_ascii_lowercase().contains("bootstrap") || report.expectation.to_ascii_lowercase().contains("socks"));
        assert!(report.summary.to_ascii_lowercase().contains("socks"));
    }
}
