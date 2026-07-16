use crate::process::hidden_command;
use crate::types::{LinkAdapterFact, LinkDuplex, NetworkInterface};
use super::parsers::parse_speed_mbps;

pub fn platform_source() -> &'static str {
    "windows_netadapter"
}

pub fn enrich_adapters(interfaces: &[NetworkInterface]) -> Result<Vec<LinkAdapterFact>, String> {
    let script = "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -or $_.Status -eq 'Disconnected' } | \
         ForEach-Object { [PSCustomObject]@{ Name = $_.Name; LinkSpeed = $_.LinkSpeed; FullDuplex = $_.FullDuplex; MediaType = $_.MediaType.ToString(); Status = $_.Status.ToString() } } | \
         ConvertTo-Json -Compress";

    let output = run_powershell(script)?;
    if output.trim().is_empty() {
        return Ok(fallback_from_env(interfaces));
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

        facts.push(LinkAdapterFact {
            name: iface.name.clone(),
            friendly_name: iface.friendly_name.clone(),
            kind: iface.kind,
            is_up: status_up || iface.is_up,
            is_default_route: iface.is_default_route,
            speed_mbps: raw_speed.as_deref().and_then(parse_speed_mbps),
            duplex,
            media,
            raw_speed,
        });
    }

    if facts.is_empty() {
        Ok(fallback_from_env(interfaces))
    } else {
        Ok(facts)
    }
}

fn fallback_from_env(interfaces: &[NetworkInterface]) -> Vec<LinkAdapterFact> {
    interfaces
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
