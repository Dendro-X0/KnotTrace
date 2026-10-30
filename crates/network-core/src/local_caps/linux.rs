//! Linux local-caps **observe** (Band A2) — no repair.

use crate::process::hidden_command;
use crate::types::NetworkInterface;
use super::parsers::{classify_linux_tcp_caps, parse_sys_power_control};

pub fn platform_available() -> bool {
    true
}

pub fn platform_can_repair() -> bool {
    false
}

pub fn platform_note() -> String {
    "Linux TCP window-scaling / buffer observe only. In-app repair stays Windows-only.".to_string()
}

pub fn read_autotuning_level() -> Result<String, String> {
    let scaling = read_sysctl_file("/proc/sys/net/ipv4/tcp_window_scaling")?;
    let rmem = read_sysctl_file("/proc/sys/net/ipv4/tcp_rmem").unwrap_or_default();
    Ok(classify_linux_tcp_caps(&scaling, &rmem))
}

pub fn set_autotuning_level(_level: &str) -> Result<(), String> {
    Err(platform_note())
}

pub fn read_adapter_power_saving(adapter_alias: &str) -> Result<Option<bool>, String> {
    let path = format!("/sys/class/net/{adapter_alias}/device/power/control");
    match std::fs::read_to_string(&path) {
        Ok(raw) => Ok(parse_sys_power_control(&raw)),
        Err(_) => {
            // Fall back to ethtool wake-on-lan / energy detect is noisy; treat as unknown.
            let _ = hidden_command("ethtool").arg(adapter_alias).output();
            Ok(None)
        }
    }
}

pub fn set_adapter_power_saving(_adapter_alias: &str, _allow_turn_off: bool) -> Result<(), String> {
    Err(platform_note())
}

pub fn resolve_adapter_alias(interfaces: &[NetworkInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| iface.name.clone())
}

fn read_sysctl_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .map_err(|error| format!("failed to read {path}: {error}"))
}
