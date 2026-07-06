use crate::types::*;
use hickory_resolver::config::{NameServerConfigGroup, ResolverConfig};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::TokioResolver;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

const INTERNET_PROBE_HOST: &str = "1.1.1.1";
const INTERNET_PROBE_PORT: u16 = 443;
const DNS_PROBE_QUERY: &str = "example.com";
const PROBE_ATTEMPTS: usize = 3;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn measure_tcp_latency(host: &str, port: u16, attempts: usize) -> Result<LatencySample, String> {
    tcp_latency_sample(
        &format!("latency:{host}:{port}"),
        host.to_string(),
        port,
        attempts,
    )
    .await
}

pub async fn run_quick_probe(environment: &EnvironmentSnapshot) -> Result<ProbeResult, String> {
    let started = Instant::now();

    let gateway = if let Some(gateway_ip) = environment.default_gateway.clone() {
        match tcp_latency_sample(&format!("gateway:{gateway_ip}"), gateway_ip.clone(), 80, PROBE_ATTEMPTS).await {
            Ok(sample) => Some(sample),
            Err(_) => {
                tcp_latency_sample(&format!("gateway:{gateway_ip}"), gateway_ip, 53, PROBE_ATTEMPTS)
                    .await
                    .ok()
            }
        }
    } else {
        None
    };

    let internet = Some(
        tcp_latency_sample(
            &format!("internet:{INTERNET_PROBE_HOST}"),
            INTERNET_PROBE_HOST.to_string(),
            INTERNET_PROBE_PORT,
            PROBE_ATTEMPTS,
        )
        .await?,
    );

    let mut dns = Vec::new();
    for server in &environment.dns_servers {
        dns.push(probe_dns_resolver(&server.address, DNS_PROBE_QUERY).await);
    }

    if dns.is_empty() {
        dns.push(probe_dns_resolver("system", DNS_PROBE_QUERY).await);
    }

    Ok(ProbeResult {
        gateway,
        internet,
        dns,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

async fn tcp_latency_sample(
    label: &str,
    host: String,
    port: u16,
    attempts: usize,
) -> Result<LatencySample, String> {
    let mut samples = Vec::new();
    let mut failures = 0usize;

    for _ in 0..attempts {
        let addr = resolve_socket_addr(&host, port).await?;
        let started = Instant::now();

        match timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
            Ok(Ok(_stream)) => samples.push(started.elapsed().as_secs_f64() * 1000.0),
            _ => failures += 1,
        }
    }

    if samples.is_empty() {
        return Err(format!("no successful TCP probes for {label}"));
    }

    let avg_ms = samples.iter().sum::<f64>() / samples.len() as f64;
    let loss_pct = (failures as f64 / attempts as f64) * 100.0;

    Ok(LatencySample {
        target: label.to_string(),
        avg_ms,
        loss_pct,
    })
}

async fn resolve_socket_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| format!("lookup failed for {host}:{port}: {error}"))?
        .next()
        .ok_or_else(|| format!("no addresses for {host}:{port}"))
}

fn build_resolver(resolver_label: &str) -> Result<TokioResolver, String> {
    if resolver_label == "system" {
        return TokioResolver::builder_tokio()
            .map_err(|error| error.to_string())
            .map(|builder| builder.build());
    }

    if let Ok(ip) = resolver_label.parse::<IpAddr>() {
        let mut config = ResolverConfig::new();
        let group = NameServerConfigGroup::from_ips_clear(&[ip], 53, true);
        for nameserver in group.iter() {
            config.add_name_server(nameserver.clone());
        }
        return Ok(
            TokioResolver::builder_with_config(config, TokioConnectionProvider::default()).build(),
        );
    }

    TokioResolver::builder_tokio()
        .map_err(|error| error.to_string())
        .map(|builder| builder.build())
}

pub async fn resolve_dns_addresses(resolver_label: &str, domain: &str) -> Result<Vec<String>, String> {
    let resolver = build_resolver(resolver_label)?;
    let lookup = resolver
        .lookup_ip(domain)
        .await
        .map_err(|error| error.to_string())?;

    let mut addresses: Vec<String> = lookup.iter().map(|ip| ip.to_string()).collect();
    addresses.sort();
    addresses.dedup();
    Ok(addresses)
}

pub async fn probe_dns_resolver(resolver_label: &str, query: &str) -> DnsProbe {
    let started = Instant::now();

    match build_resolver(resolver_label) {
        Ok(resolver) => match resolver.lookup_ip(query).await {
            Ok(_lookup) => DnsProbe {
                resolver: resolver_label.to_string(),
                query: query.to_string(),
                latency_ms: started.elapsed().as_secs_f64() * 1000.0,
                success: true,
            },
            Err(error) => {
                tracing::debug!(target = "network_core::probe", "dns probe failed: {error}");
                DnsProbe {
                    resolver: resolver_label.to_string(),
                    query: query.to_string(),
                    latency_ms: started.elapsed().as_secs_f64() * 1000.0,
                    success: false,
                }
            }
        },
        Err(error) => {
            tracing::debug!(target = "network_core::probe", "dns resolver init failed: {error}");
            DnsProbe {
                resolver: resolver_label.to_string(),
                query: query.to_string(),
                latency_ms: started.elapsed().as_secs_f64() * 1000.0,
                success: false,
            }
        }
    }
}
