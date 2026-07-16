use crate::process::hidden_command;
use crate::types::NetworkInterface;
use super::parsers::{
    macos_dns_is_dhcp, macos_parse_dns_servers, macos_service_for_device, parse_macos_hardware_ports,
};
use std::path::Path;

const NETWORKSETUP: &str = "/usr/sbin/networksetup";

pub fn can_apply() -> bool {
    Path::new(NETWORKSETUP).exists()
}

pub fn platform_note() -> String {
    "DNS changes apply to the active macOS network service via networksetup. Administrator approval may be required.".to_string()
}

pub fn dns_target_for_interface(iface: &NetworkInterface) -> Option<String> {
    if let Some(friendly) = iface.friendly_name.as_deref().filter(|name| !name.trim().is_empty())
    {
        if let Ok(service) = resolve_network_service(friendly) {
            return Some(service);
        }
        return Some(friendly.to_string());
    }

    resolve_network_service(&iface.name)
        .ok()
        .or_else(|| Some(iface.name.clone()))
}

pub fn read_dns_config(interface_alias: &str) -> Result<(Vec<String>, bool), String> {
    let service = resolve_network_service(interface_alias)?;
    let output = run_networksetup(&["-getdnsservers", &service])?;
    let use_dhcp = macos_dns_is_dhcp(&output);
    let servers = macos_parse_dns_servers(&output);
    Ok((servers, use_dhcp))
}

pub fn set_static_dns(interface_alias: &str, resolver: &str) -> Result<(), String> {
    let service = resolve_network_service(interface_alias)?;
    run_networksetup(&["-setdnsservers", &service, resolver]).map(|_| ())
}

pub fn restore_dns_config(
    interface_alias: &str,
    previous_servers: &[String],
    use_dhcp: bool,
) -> Result<(), String> {
    let service = resolve_network_service(interface_alias)?;
    if use_dhcp || previous_servers.is_empty() {
        return run_networksetup(&["-setdnsservers", &service, "empty"]).map(|_| ());
    }

    let mut args = vec!["-setdnsservers".to_string(), service];
    args.extend(previous_servers.iter().cloned());
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    run_networksetup(&args_ref).map(|_| ())
}

fn resolve_network_service(alias: &str) -> Result<String, String> {
    let ports_output = run_networksetup(&["-listallhardwareports"])?;
    let pairs = parse_macos_hardware_ports(&ports_output);

    if pairs.iter().any(|(port, _)| port == alias) {
        return Ok(alias.to_string());
    }

    if let Some(service) = macos_service_for_device(&pairs, alias) {
        return Ok(service);
    }

    // Soft match: alias might be a partial friendly name.
    if let Some((port, _)) = pairs
        .iter()
        .find(|(port, _)| port.eq_ignore_ascii_case(alias))
    {
        return Ok(port.clone());
    }

    Err(format!(
        "Could not map '{alias}' to a macOS network service. Available: {}",
        pairs
            .iter()
            .map(|(port, device)| format!("{port} ({device})"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn run_networksetup(args: &[&str]) -> Result<String, String> {
    let output = hidden_command(NETWORKSETUP)
        .args(args)
        .output()
        .map_err(|error| format!("failed to launch networksetup: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(if detail.is_empty() {
            "networksetup failed. Try approving the admin prompt or granting privileges.".to_string()
        } else {
            detail
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
