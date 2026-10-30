//! macOS local-caps **observe** (Band A2) — no repair.

use crate::process::hidden_command;
use crate::types::NetworkInterface;
use super::parsers::classify_macos_tcp_caps;

pub fn platform_available() -> bool {
    true
}

pub fn platform_can_repair() -> bool {
    false
}

pub fn platform_note() -> String {
    "macOS TCP auto-buffer observe only (sysctl). In-app repair stays Windows-only.".to_string()
}

pub fn read_autotuning_level() -> Result<String, String> {
    let do_rcv = sysctl_n("net.inet.tcp.doautorcvbuf").unwrap_or_else(|_| "1".to_string());
    let do_snd = sysctl_n("net.inet.tcp.doautosndbuf").unwrap_or_else(|_| "1".to_string());
    Ok(classify_macos_tcp_caps(&do_rcv, &do_snd))
}

pub fn set_autotuning_level(_level: &str) -> Result<(), String> {
    Err(platform_note())
}

pub fn read_adapter_power_saving(_adapter_alias: &str) -> Result<Option<bool>, String> {
    // Energy Saver / pmset does not map cleanly to per-NIC “turn off device”; leave unknown.
    Ok(None)
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

fn sysctl_n(key: &str) -> Result<String, String> {
    let output = hidden_command("sysctl")
        .args(["-n", key])
        .output()
        .map_err(|error| format!("failed to launch sysctl: {error}"))?;
    if !output.status.success() {
        return Err(format!("sysctl {key} failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
