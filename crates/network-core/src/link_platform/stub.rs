use crate::types::{LinkAdapterFact, NetworkInterface};

pub fn platform_source() -> &'static str {
    "none"
}

pub fn enrich_adapters(interfaces: &[NetworkInterface]) -> Result<Vec<LinkAdapterFact>, String> {
    Ok(interfaces
        .iter()
        .map(|iface| LinkAdapterFact {
            name: iface.name.clone(),
            friendly_name: iface.friendly_name.clone(),
            kind: iface.kind,
            is_up: iface.is_up,
            is_default_route: iface.is_default_route,
            speed_mbps: None,
            duplex: None,
            media: None,
            raw_speed: None,
        })
        .collect())
}
