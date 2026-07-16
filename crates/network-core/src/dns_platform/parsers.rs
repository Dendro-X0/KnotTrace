//! Shared parsing helpers for Unix DNS platform backends.
//! Kept cfg-free so unit tests run on all CI hosts (including Windows).

#![allow(dead_code)] // linux/macos backends consume these; Windows builds still run parser tests.

/// Parse `networksetup -listallhardwareports` into (service/hardware port name, device) pairs.
pub fn parse_macos_hardware_ports(output: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut current_port: Option<String> = None;

    for line in output.lines() {
        let line = line.trim();
        if let Some(port) = line.strip_prefix("Hardware Port: ") {
            current_port = Some(port.to_string());
            continue;
        }
        if let Some(device) = line.strip_prefix("Device: ") {
            if let Some(port) = current_port.take() {
                if !device.is_empty() {
                    pairs.push((port, device.to_string()));
                }
            }
        }
    }

    pairs
}

/// Map a BSD device name (e.g. `en0`) to a macOS network service name (e.g. `Wi-Fi`).
pub fn macos_service_for_device(hardware_ports: &[(String, String)], device: &str) -> Option<String> {
    hardware_ports
        .iter()
        .find(|(_, dev)| dev.eq_ignore_ascii_case(device))
        .map(|(port, _)| port.clone())
}

/// True when `networksetup -getdnsservers` reports DHCP / no manual servers.
pub fn macos_dns_is_dhcp(output: &str) -> bool {
    let trimmed = output.trim();
    trimmed.is_empty()
        || trimmed.contains("aren't any DNS Servers")
        || trimmed.contains("There aren't any DNS Servers")
}

pub fn macos_parse_dns_servers(output: &str) -> Vec<String> {
    if macos_dns_is_dhcp(output) {
        return Vec::new();
    }
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.contains("aren't any DNS Servers"))
        .map(str::to_string)
        .collect()
}

/// Parse `nmcli -g IP4.DNS device show` output (newline or `|`-separated).
pub fn parse_nmcli_dns_list(output: &str) -> Vec<String> {
    output
        .split(|ch: char| ch.is_whitespace() || ch == '|' || ch == ',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Parse `resolvectl dns <iface>` lines for server addresses.
pub fn parse_resolvectl_dns(output: &str) -> Vec<String> {
    let mut servers = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        // Formats:
        // Link 2 (wlan0): 1.1.1.1 8.8.8.8
        // or: DNS Servers: 1.1.1.1
        if let Some((_, rhs)) = line.split_once(':') {
            for part in rhs.split_whitespace() {
                let cleaned = part.trim().trim_matches('%').to_string();
                // Skip scope suffixes like fe80::1%wlan0 — keep only if it looks like an IP
                let candidate = cleaned.split('%').next().unwrap_or(&cleaned);
                if looks_like_ip(candidate) {
                    servers.push(candidate.to_string());
                }
            }
        }
    }
    servers
}

fn looks_like_ip(value: &str) -> bool {
    if value.parse::<std::net::Ipv4Addr>().is_ok() {
        return true;
    }
    value.parse::<std::net::Ipv6Addr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_macos_hardware_ports() {
        let output = "\
Hardware Port: Wi-Fi
Device: en0
Ethernet Address: aa:bb

Hardware Port: Ethernet
Device: en1
Ethernet Address: cc:dd
";
        let pairs = parse_macos_hardware_ports(output);
        assert_eq!(
            pairs,
            vec![
                ("Wi-Fi".to_string(), "en0".to_string()),
                ("Ethernet".to_string(), "en1".to_string()),
            ]
        );
        assert_eq!(
            macos_service_for_device(&pairs, "en0").as_deref(),
            Some("Wi-Fi")
        );
    }

    #[test]
    fn detects_macos_dhcp_dns_message() {
        assert!(macos_dns_is_dhcp(
            "There aren't any DNS Servers set on Wi-Fi."
        ));
        assert!(macos_parse_dns_servers("There aren't any DNS Servers set on Wi-Fi.").is_empty());
        assert_eq!(
            macos_parse_dns_servers("1.1.1.1\n1.0.0.1\n"),
            vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()]
        );
    }

    #[test]
    fn parses_nmcli_and_resolvectl_dns() {
        assert_eq!(
            parse_nmcli_dns_list("1.1.1.1\n8.8.8.8\n"),
            vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()]
        );
        assert_eq!(
            parse_resolvectl_dns("Link 3 (wlan0): 9.9.9.9 149.112.112.112\n"),
            vec!["9.9.9.9".to_string(), "149.112.112.112".to_string()]
        );
    }
}
