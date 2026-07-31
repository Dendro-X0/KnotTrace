use crate::process::hidden_command;
use crate::types::{LinkAdapterFact, LinkDuplex, LinkKind, NetworkInterface};
use super::parsers::{parse_netsh_wlan_interfaces, parse_speed_mbps};

pub fn platform_source() -> &'static str {
    "windows_netadapter+wlan"
}

pub fn enrich_adapters(interfaces: &[NetworkInterface]) -> Result<Vec<LinkAdapterFact>, String> {
    let script = "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -or $_.Status -eq 'Disconnected' } | \
         ForEach-Object { [PSCustomObject]@{ Name = $_.Name; LinkSpeed = $_.LinkSpeed; FullDuplex = $_.FullDuplex; MediaType = $_.MediaType.ToString(); Status = $_.Status.ToString() } } | \
         ConvertTo-Json -Compress";

    let output = run_powershell(script)?;
    if output.trim().is_empty() {
        return Ok(merge_wifi(fallback_from_env(interfaces)));
    }

    let value: serde_json::Value = serde_json::from_str(&output)
        .map_err(|error| format!("invalid Get-NetAdapter JSON: {error}"))?;
    let rows = match value {
        serde_json::Value::Array(items) => items,
        other => vec![other],
    };

    let mut facts = Vec::new();
    for iface in interfaces {
        let alias = iface
            .friendly_name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or(iface.name.as_str());

        let row = rows.iter().find(|row| {
            row.get("Name")
                .and_then(|v| v.as_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(alias))
        });

        let (raw_speed, duplex, media, status_up) = if let Some(row) = row {
            let raw_speed = row
                .get("LinkSpeed")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let duplex = row.get("FullDuplex").and_then(|v| match v {
                serde_json::Value::Bool(true) => Some(LinkDuplex::Full),
                serde_json::Value::Bool(false) => Some(LinkDuplex::Half),
                _ => None,
            });
            let media = row
                .get("MediaType")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let status_up = row
                .get("Status")
                .and_then(|v| v.as_str())
                .is_some_and(|status| status.eq_ignore_ascii_case("Up"));
            (raw_speed, duplex, media, status_up)
        } else {
            (None, None, None, iface.is_up)
        };

        facts.push(base_fact(
            iface,
            status_up || iface.is_up,
            raw_speed.as_deref().and_then(parse_speed_mbps),
            duplex,
            media,
            raw_speed,
        ));
    }

    let facts = if facts.is_empty() {
        fallback_from_env(interfaces)
    } else {
        facts
    };
    Ok(merge_wifi(facts))
}

fn merge_wifi(mut facts: Vec<LinkAdapterFact>) -> Vec<LinkAdapterFact> {
    let output = hidden_command("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok();
    let Some(output) = output else {
        return facts;
    };
    if !output.status.success() {
        return facts;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let wifi_rows = parse_netsh_wlan_interfaces(&text);
    if wifi_rows.is_empty() {
        return facts;
    }

    for fact in &mut facts {
        if fact.kind != LinkKind::WiFi {
            continue;
        }
        let alias = fact
            .friendly_name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or(fact.name.as_str());
        let Some(wifi) = wifi_rows.iter().find(|row| {
            row.name.eq_ignore_ascii_case(alias) || row.name.eq_ignore_ascii_case(&fact.name)
        }) else {
            // Single associated interface: apply to first Wi-Fi fact.
            if wifi_rows.len() == 1 {
                apply_wifi(fact, &wifi_rows[0]);
            }
            continue;
        };
        apply_wifi(fact, wifi);
    }
    facts
}

fn apply_wifi(fact: &mut LinkAdapterFact, wifi: &super::parsers::WifiInterfaceSnapshot) {
    fact.wifi_ssid = wifi.ssid.clone();
    fact.wifi_signal_percent = wifi.signal_percent;
    fact.wifi_signal_dbm = wifi.signal_dbm;
    fact.wifi_channel = wifi.channel;
    fact.wifi_band = wifi.band;
    fact.wifi_radio = wifi.radio.clone();
    if fact.speed_mbps.is_none() {
        fact.speed_mbps = wifi.receive_rate_mbps;
    }
    if fact.raw_speed.is_none() {
        if let Some(rate) = wifi.receive_rate_mbps {
            fact.raw_speed = Some(format!("{rate} Mbps"));
        }
    }
}

fn base_fact(
    iface: &NetworkInterface,
    is_up: bool,
    speed_mbps: Option<u32>,
    duplex: Option<LinkDuplex>,
    media: Option<String>,
    raw_speed: Option<String>,
) -> LinkAdapterFact {
    LinkAdapterFact {
        name: iface.name.clone(),
        friendly_name: iface.friendly_name.clone(),
        kind: iface.kind,
        is_up,
        is_default_route: iface.is_default_route,
        speed_mbps,
        duplex,
        media,
        raw_speed,
        wifi_ssid: None,
        wifi_signal_percent: None,
        wifi_signal_dbm: None,
        wifi_channel: None,
        wifi_band: None,
        wifi_radio: None,
    }
}

fn fallback_from_env(interfaces: &[NetworkInterface]) -> Vec<LinkAdapterFact> {
    interfaces
        .iter()
        .map(|iface| base_fact(iface, iface.is_up, None, None, None, None))
        .collect()
}

fn run_powershell(script: &str) -> Result<String, String> {
    let output = hidden_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|error| format!("failed to launch PowerShell: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Get-NetAdapter failed".to_string()
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
