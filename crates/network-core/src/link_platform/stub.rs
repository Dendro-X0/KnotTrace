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
            wifi_ssid: None,
            wifi_signal_percent: None,
            wifi_signal_dbm: None,
            wifi_channel: None,
            wifi_band: None,
            wifi_radio: None,
            wifi_noise_dbm: None,
            wifi_tx_retries: None,
        })
        .collect())
}
