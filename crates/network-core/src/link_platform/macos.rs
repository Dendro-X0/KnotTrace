use crate::process::hidden_command;
use crate::types::{LinkAdapterFact, LinkDuplex, NetworkInterface};
use super::parsers::{parse_duplex, parse_speed_mbps};
use std::path::Path;

const NETWORKSETUP: &str = "/usr/sbin/networksetup";

pub fn platform_source() -> &'static str {
    "macos_networksetup"
}

pub fn enrich_adapters(interfaces: &[NetworkInterface]) -> Result<Vec<LinkAdapterFact>, String> {
    let ports = list_hardware_ports().unwrap_or_default();
    let mut facts = Vec::new();

    for iface in interfaces {
        let service = resolve_service(&ports, iface);
        let media_output = service
            .as_deref()
            .and_then(|name| run_networksetup(&["-getMedia", name]).ok());
        let (raw_speed, duplex, media) = media_output
            .as_deref()
            .map(parse_get_media)
            .unwrap_or((None, None, None));

        facts.push(LinkAdapterFact {
            name: iface.name.clone(),
            friendly_name: iface.friendly_name.clone().or(service),
            kind: iface.kind,
            is_up: iface.is_up,
            is_default_route: iface.is_default_route,
            speed_mbps: raw_speed.as_deref().and_then(parse_speed_mbps),
            duplex,
            media,
            raw_speed,
        });
    }

    Ok(facts)
}

fn list_hardware_ports() -> Result<Vec<(String, String)>, String> {
    if !Path::new(NETWORKSETUP).exists() {
        return Err("networksetup missing".to_string());
    }
    let output = run_networksetup(&["-listallhardwareports"])?;
    Ok(parse_hardware_ports(&output))
}

fn parse_hardware_ports(output: &str) -> Vec<(String, String)> {
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

fn resolve_service(ports: &[(String, String)], iface: &NetworkInterface) -> Option<String> {
    if let Some(friendly) = iface.friendly_name.as_deref() {
        if ports.iter().any(|(port, _)| port == friendly) {
            return Some(friendly.to_string());
        }
    }
    ports
        .iter()
        .find(|(_, device)| device.eq_ignore_ascii_case(&iface.name))
        .map(|(port, _)| port.clone())
}

fn parse_get_media(output: &str) -> (Option<String>, Option<LinkDuplex>, Option<String>) {
    let active = output
        .lines()
        .find_map(|line| line.trim().strip_prefix("Active: "))
        .or_else(|| {
            output
                .lines()
                .find_map(|line| line.trim().strip_prefix("Current: "))
        })
        .unwrap_or(output.trim());

    let media = Some(active.to_string());
    let duplex = parse_duplex(active);
    let lower = active.to_ascii_lowercase();
    let raw_speed = if lower.contains("1000baset") || lower.contains("1gbase") {
        Some("1000 Mbps".to_string())
    } else if lower.contains("100baset") {
        Some("100 Mbps".to_string())
    } else if lower.contains("10baset") && !lower.contains("100") && !lower.contains("1000") {
        Some("10 Mbps".to_string())
    } else if lower.contains("2500") {
        Some("2500 Mbps".to_string())
    } else if lower.contains("10g") {
        Some("10000 Mbps".to_string())
    } else {
        None
    };

    (raw_speed, duplex, media)
}

fn run_networksetup(args: &[&str]) -> Result<String, String> {
    let output = hidden_command(NETWORKSETUP)
        .args(args)
        .output()
        .map_err(|error| format!("failed to launch networksetup: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "networksetup -getMedia failed".to_string()
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
