use crate::process::hidden_command;
use crate::types::{LinkAdapterFact, LinkDuplex, LinkKind, NetworkInterface};
use super::parsers::{parse_duplex, parse_iw_link, parse_speed_mbps};
use std::fs;
use std::path::PathBuf;

pub fn platform_source() -> &'static str {
    "linux_sysfs+iw"
}

pub fn enrich_adapters(interfaces: &[NetworkInterface]) -> Result<Vec<LinkAdapterFact>, String> {
    let mut facts = Vec::new();
    for iface in interfaces {
        let base = PathBuf::from("/sys/class/net").join(&iface.name);
        let raw_speed = read_trimmed(base.join("speed"));
        let speed_mbps = raw_speed.as_deref().and_then(|raw| {
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

        let mut fact = LinkAdapterFact {
            name: iface.name.clone(),
            friendly_name: iface.friendly_name.clone(),
            kind: iface.kind,
            is_up,
            is_default_route: iface.is_default_route,
            speed_mbps,
            duplex,
            media: operstate,
            raw_speed,
            wifi_ssid: None,
            wifi_signal_percent: None,
            wifi_signal_dbm: None,
            wifi_channel: None,
            wifi_band: None,
            wifi_radio: None,
        };

        if iface.kind == LinkKind::WiFi {
            if let Ok(output) = hidden_command("iw")
                .args(["dev", &iface.name, "link"])
                .output()
            {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let wifi = parse_iw_link(&text);
                    fact.wifi_ssid = wifi.ssid;
                    fact.wifi_signal_percent = wifi.signal_percent;
                    fact.wifi_signal_dbm = wifi.signal_dbm;
                    fact.wifi_channel = wifi.channel;
                    fact.wifi_band = wifi.band;
                    fact.wifi_radio = wifi.radio;
                }
            }
        }

        facts.push(fact);
    }
    Ok(facts)
}

fn read_trimmed(path: PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
