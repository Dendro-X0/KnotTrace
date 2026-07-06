use crate::types::EnvironmentSnapshot;

/// Stable fingerprint of the network environment for change detection.
pub fn environment_fingerprint(environment: &EnvironmentSnapshot) -> String {
    let mut dns = environment
        .dns_servers
        .iter()
        .map(|server| server.address.as_str())
        .collect::<Vec<_>>();
    dns.sort_unstable();

    let mut tags = environment
        .tags
        .iter()
        .map(|tag| format!("{tag:?}"))
        .collect::<Vec<_>>();
    tags.sort_unstable();

    format!(
        "gw={}|iface={}|proxy={}|tor={}|dns={}|tags={}",
        environment.default_gateway.as_deref().unwrap_or("none"),
        environment.active_interface.as_deref().unwrap_or("none"),
        if environment.proxy.enabled {
            environment.proxy.server.as_deref().unwrap_or("enabled")
        } else {
            "off"
        },
        if environment.tor.detected {
            environment
                .tor
                .socks_endpoint
                .as_deref()
                .unwrap_or("detected")
        } else {
            "off"
        },
        dns.join(","),
        tags.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{default_tor_status, DnsServer, EnvironmentTag, ProxySettings};

    #[test]
    fn fingerprint_is_stable_for_same_environment() {
        let environment = EnvironmentSnapshot {
            hostname: "host".to_string(),
            interfaces: Vec::new(),
            dns_servers: vec![DnsServer {
                address: "1.1.1.1".to_string(),
            }],
            proxy: ProxySettings {
                enabled: false,
                server: None,
                source: "none".to_string(),
            },
            tags: vec![EnvironmentTag::HomeLan],
            default_gateway: Some("192.168.1.1".to_string()),
            active_interface: Some("eth0".to_string()),
            tor: default_tor_status(),
        };

        let first = environment_fingerprint(&environment);
        let second = environment_fingerprint(&environment);
        assert_eq!(first, second);
    }
}
