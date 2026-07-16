use crate::types::{LinkAdapterFact, LinkDuplex, NetworkInterface};
use super::parsers::{parse_duplex, parse_speed_mbps};
use std::fs;
use std::path::PathBuf;

pub fn platform_source() -> &'static str {
    "linux_sysfs"
}

pub fn enrich_adapters(interfaces: &[NetworkInterface]) -> Result<Vec<LinkAdapterFact>, String> {
    let mut facts = Vec::new();
    for iface in interfaces {
        let base = PathBuf::from("/sys/class/net").join(&iface.name);
        let raw_speed = read_trimmed(base.join("speed"));
        let speed_mbps = raw_speed.as_deref().and_then(|raw| {
            // sysfs uses -1 when unknown (common on Wi-Fi)
            if raw == "-1" {
                None
            } else {
                parse_speed_mbps(raw)
            }
        });
        let duplex = read_trimmed(base.join("duplex"))
            .as_deref()
            .and_then(parse_duplex)
            .or(Some(LinkDuplex::Unknown));
        let operstate = read_trimmed(base.join("operstate"));
        let is_up = operstate
            .as_deref()
            .is_some_and(|state| state.eq_ignore_ascii_case("up"))
            || iface.is_up;

        facts.push(LinkAdapterFact {
            name: iface.name.clone(),
            friendly_name: iface.friendly_name.clone(),
            kind: iface.kind,
            is_up,
            is_default_route: iface.is_default_route,
            speed_mbps,
            duplex,
            media: operstate,
            raw_speed,
        });
    }
    Ok(facts)
}

fn read_trimmed(path: PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
