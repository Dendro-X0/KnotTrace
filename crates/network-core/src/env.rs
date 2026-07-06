use crate::types::*;
use std::collections::HashSet;

const VPN_NAME_HINTS: &[&str] = &[
    "tun", "tap", "wg", "wireguard", "nordlynx", "openvpn", "wintun", "tailscale",
    "zerotier", "utun", "ppp", "ipsec", "nordvpn", "proton", "mullvad",
];

pub fn detect_environment() -> Result<EnvironmentSnapshot, String> {
    let raw_interfaces = netdev::get_interfaces();
    let interfaces = collect_interfaces(&raw_interfaces);
    let dns_servers = collect_dns_servers(&raw_interfaces);
    let proxy = detect_proxy();
    let tor = crate::tor::detect_tor_status(&proxy, &interfaces);
    let default_gateway = pick_default_gateway(&interfaces);
    let active_interface = pick_active_interface(&interfaces);

    let mut tags = infer_tags(&interfaces, &proxy, &tor, default_gateway.as_deref());
    tags.sort_by_key(|tag| format!("{tag:?}"));
    tags.dedup();

    Ok(EnvironmentSnapshot {
        hostname: hostname(),
        interfaces,
        dns_servers,
        proxy,
        tags,
        default_gateway,
        active_interface,
        tor,
    })
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn collect_interfaces(raw: &[netdev::Interface]) -> Vec<NetworkInterface> {
    let default_name = netdev::get_default_interface()
        .ok()
        .map(|iface| iface.name);

    raw.iter()
        .map(|iface| {
            let ipv4 = iface
                .ipv4
                .iter()
                .map(|addr| addr.addr().to_string())
                .collect::<Vec<_>>();

            let gateway = iface
                .gateway
                .as_ref()
                .and_then(|device| device.ipv4.first())
                .map(|addr| addr.to_string());

            NetworkInterface {
                name: iface.name.clone(),
                friendly_name: iface.friendly_name.clone(),
                kind: map_link_kind(iface),
                is_up: !iface.ipv4.is_empty(),
                ipv4,
                gateway,
                is_default_route: default_name
                    .as_ref()
                    .is_some_and(|name| name == &iface.name),
            }
        })
        .filter(|iface| iface.kind != LinkKind::Loopback)
        .collect()
}

fn map_link_kind(iface: &netdev::Interface) -> LinkKind {
    use netdev::interface::InterfaceType;

    let name = iface.name.to_lowercase();
    let friendly = iface
        .friendly_name
        .as_deref()
        .unwrap_or("")
        .to_lowercase();

    match iface.if_type {
        InterfaceType::Loopback => LinkKind::Loopback,
        InterfaceType::Wireless80211 => LinkKind::WiFi,
        InterfaceType::Wwanpp | InterfaceType::Wwanpp2 | InterfaceType::Wman => {
            LinkKind::Cellular
        }
        InterfaceType::Ethernet
        | InterfaceType::GigabitEthernet
        | InterfaceType::FastEthernetFx
        | InterfaceType::FastEthernetT => LinkKind::Ethernet,
        InterfaceType::Tunnel | InterfaceType::Ppp => LinkKind::Virtual,
        _ if is_virtual_by_name(&name, &friendly) => LinkKind::Virtual,
        _ => LinkKind::Unknown,
    }
}

fn is_virtual_by_name(name: &str, friendly: &str) -> bool {
    VPN_NAME_HINTS
        .iter()
        .any(|hint| name.contains(hint) || friendly.contains(hint))
        || friendly.contains("virtual")
        || friendly.contains("vpn")
}

fn pick_default_gateway(interfaces: &[NetworkInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .and_then(|iface| iface.gateway.clone())
        .or_else(|| {
            interfaces
                .iter()
                .find(|iface| iface.is_up && iface.gateway.is_some())
                .and_then(|iface| iface.gateway.clone())
        })
}

fn pick_active_interface(interfaces: &[NetworkInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .map(|iface| iface.name.clone())
        .or_else(|| {
            interfaces
                .iter()
                .find(|iface| iface.is_up && !iface.ipv4.is_empty())
                .map(|iface| iface.name.clone())
        })
}

fn collect_dns_servers(raw: &[netdev::Interface]) -> Vec<DnsServer> {
    let mut servers = Vec::new();
    let mut seen = HashSet::new();

    for iface in raw {
        for ip in &iface.dns_servers {
            if !is_probeable_dns(ip) {
                continue;
            }
            let address = ip.to_string();
            if seen.insert(address.clone()) {
                servers.push(DnsServer { address });
            }
        }
    }

    if servers.is_empty() {
        if let Ok((config, _)) = hickory_resolver::system_conf::read_system_conf() {
            for nameserver in config.name_servers() {
                let ip = nameserver.socket_addr.ip();
                if !is_probeable_dns(&ip) {
                    continue;
                }
                let address = ip.to_string();
                if seen.insert(address.clone()) {
                    servers.push(DnsServer { address });
                }
            }
        }
    }

    servers
}

fn is_probeable_dns(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(addr) => !addr.is_unspecified() && !addr.is_loopback(),
        std::net::IpAddr::V6(addr) => {
            !addr.is_unspecified()
                && !addr.is_loopback()
                && !addr.is_multicast()
                && addr.segments()[0] != 0xfe80
                && !addr.octets().starts_with(&[0xfe, 0xc0, 0x00, 0x00])
        }
    }
}

fn detect_proxy() -> ProxySettings {
    for (key, source) in [
        ("ALL_PROXY", "env:ALL_PROXY"),
        ("HTTPS_PROXY", "env:HTTPS_PROXY"),
        ("HTTP_PROXY", "env:HTTP_PROXY"),
    ] {
        if let Ok(value) = std::env::var(key) {
            if !value.trim().is_empty() {
                return ProxySettings {
                    enabled: true,
                    server: Some(value),
                    source: source.to_string(),
                };
            }
        }
    }

    #[cfg(windows)]
    {
        if let Some(proxy) = detect_windows_system_proxy() {
            return proxy;
        }
    }

    ProxySettings {
        enabled: false,
        server: None,
        source: "none".to_string(),
    }
}

#[cfg(windows)]
fn detect_windows_system_proxy() -> Option<ProxySettings> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;

    let enabled: u32 = settings.get_value("ProxyEnable").unwrap_or(0);
    if enabled == 0 {
        return None;
    }

    let server: String = settings.get_value("ProxyServer").ok()?;
    if server.trim().is_empty() {
        return None;
    }

    Some(ProxySettings {
        enabled: true,
        server: Some(server),
        source: "windows:internet_settings".to_string(),
    })
}

#[cfg(not(windows))]
fn detect_windows_system_proxy() -> Option<ProxySettings> {
    None
}

fn infer_tags(
    interfaces: &[NetworkInterface],
    proxy: &ProxySettings,
    tor: &TorStatus,
    default_gateway: Option<&str>,
) -> Vec<EnvironmentTag> {
    let mut tags = Vec::new();

    if tor.detected {
        tags.push(EnvironmentTag::Tor);
    }

    let has_vpn = interfaces.iter().any(|iface| {
        iface.kind == LinkKind::Virtual
            || VPN_NAME_HINTS.iter().any(|hint| {
                iface.name.to_lowercase().contains(hint)
                    || iface
                        .friendly_name
                        .as_ref()
                        .is_some_and(|name| name.to_lowercase().contains(hint))
            })
    });

    if has_vpn {
        tags.push(EnvironmentTag::Vpn);
    }

    if proxy.enabled {
        tags.push(EnvironmentTag::Proxy);
    }

    if let Some(gateway) = default_gateway {
        if is_private_ipv4(gateway) {
            tags.push(EnvironmentTag::HomeLan);
        }
    }

    let active_kinds = interfaces
        .iter()
        .filter(|iface| iface.is_up)
        .map(|iface| iface.kind)
        .collect::<Vec<_>>();

    if active_kinds.contains(&LinkKind::Cellular) && !tags.contains(&EnvironmentTag::HomeLan) {
        tags.push(EnvironmentTag::Public);
    }

    if tags.is_empty() {
        tags.push(EnvironmentTag::Unknown);
    }

    tags
}

fn is_private_ipv4(address: &str) -> bool {
    address.starts_with("10.")
        || address.starts_with("192.168.")
        || address.starts_with("127.")
        || address
            .strip_prefix("172.")
            .and_then(|rest| rest.split('.').next())
            .and_then(|octet| octet.parse::<u8>().ok())
            .is_some_and(|second| (16..=31).contains(&second))
}
