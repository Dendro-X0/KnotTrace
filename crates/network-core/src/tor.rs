use crate::types::*;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const TOR_SOCKS_PORTS: &[u16] = &[9050, 9150];
const TOR_NAME_HINTS: &[&str] = &["tor", "obfs4", "snowflake"];

pub fn is_tor_socks_endpoint(server: &str) -> bool {
    let lower = server.to_ascii_lowercase();
    if !lower.contains("socks") {
        return false;
    }

    TOR_SOCKS_PORTS
        .iter()
        .any(|port| lower.contains(&format!(":{port}")))
}

pub fn detect_tor_status(proxy: &ProxySettings, interfaces: &[NetworkInterface]) -> TorStatus {
    if let Some(server) = &proxy.server {
        if is_tor_socks_endpoint(server) {
            let reachable = tor_endpoint_reachable(server);
            return TorStatus {
                detected: true,
                socks_endpoint: Some(server.clone()),
                socks_reachable: reachable,
                source: "proxy_settings".to_string(),
            };
        }
    }

    for port in TOR_SOCKS_PORTS {
        if is_local_port_open(*port) {
            return TorStatus {
                detected: true,
                socks_endpoint: Some(format!("socks5h://127.0.0.1:{port}")),
                socks_reachable: true,
                source: "local_socks_port".to_string(),
            };
        }
    }

    if interfaces.iter().any(|iface| {
        let name = iface.name.to_ascii_lowercase();
        let friendly = iface
            .friendly_name
            .as_deref()
            .unwrap_or("")
            .to_ascii_lowercase();
        TOR_NAME_HINTS
            .iter()
            .any(|hint| name.contains(hint) || friendly.contains(hint))
    }) {
        return TorStatus {
            detected: true,
            socks_endpoint: None,
            socks_reachable: false,
            source: "interface_name".to_string(),
        };
    }

    default_tor_status()
}

fn tor_endpoint_reachable(server: &str) -> bool {
    if let Some(port) = parse_socks_port(server) {
        return is_local_port_open(port);
    }
    false
}

fn parse_socks_port(server: &str) -> Option<u16> {
    let trimmed = server.trim();
    let without_scheme = trimmed
        .split("://")
        .nth(1)
        .unwrap_or(trimmed);
    let host_port = without_scheme.rsplit('@').next().unwrap_or(without_scheme);
    host_port
        .rsplit(':')
        .next()
        .and_then(|port| port.parse::<u16>().ok())
}

fn is_local_port_open(port: u16) -> bool {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&address, Duration::from_millis(350)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_tor_socks_proxy_strings() {
        assert!(is_tor_socks_endpoint("socks5h://127.0.0.1:9050"));
        assert!(is_tor_socks_endpoint("socks5://127.0.0.1:9150"));
        assert!(!is_tor_socks_endpoint("http://127.0.0.1:7890"));
    }

    #[test]
    fn detects_tor_from_proxy_settings() {
        let status = detect_tor_status(
            &ProxySettings {
                enabled: true,
                server: Some("socks5h://127.0.0.1:9050".to_string()),
                source: "test".to_string(),
            },
            &[],
        );
        assert!(status.detected);
        assert_eq!(status.source, "proxy_settings");
    }
}
