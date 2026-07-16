use crate::process::hidden_command;
use crate::types::EnvironmentSnapshot;
use std::path::Path;

const NETWORKSETUP: &str = "/usr/sbin/networksetup";

pub fn platform_available() -> bool {
    Path::new(NETWORKSETUP).exists()
}

pub fn platform_note() -> String {
    "macOS interface MTU via networksetup. Administrator approval may be required.".to_string()
}

pub fn resolve_interface_name(environment: &EnvironmentSnapshot) -> Option<String> {
    environment
        .interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| environment.interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| {
            iface
                .friendly_name
                .clone()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| iface.name.clone())
        })
}

pub fn read_mtu(service_or_alias: &str) -> Result<u16, String> {
    let service = resolve_network_service(service_or_alias)?;
    let output = run_networksetup(&["-getMTU", &service])?;
    parse_networksetup_mtu(&output)
        .ok_or_else(|| format!("Could not parse MTU for `{service}`."))
}

pub fn set_mtu(service_or_alias: &str, mtu: u16) -> Result<(), String> {
    let service = resolve_network_service(service_or_alias)?;
    run_networksetup(&["-setMTU", &service, &mtu.to_string()]).map(|_| ())
}

fn resolve_network_service(alias: &str) -> Result<String, String> {
    // Prefer exact service name; fall back to alias as-is for Wi-Fi / Ethernet services.
    let ports = run_networksetup(&["-listallhardwareports"])?;
    for block in ports.split("\n\n") {
        let mut hardware = None;
        let mut device = None;
        for line in block.lines() {
            if let Some(rest) = line.strip_prefix("Hardware Port: ") {
                hardware = Some(rest.trim().to_string());
            }
            if let Some(rest) = line.strip_prefix("Device: ") {
                device = Some(rest.trim().to_string());
            }
        }
        if let Some(port) = hardware {
            if port.eq_ignore_ascii_case(alias)
                || device.as_deref().is_some_and(|dev| dev.eq_ignore_ascii_case(alias))
            {
                return Ok(port);
            }
        }
    }
    Ok(alias.to_string())
}

fn run_networksetup(args: &[&str]) -> Result<String, String> {
    let output = hidden_command(NETWORKSETUP)
        .args(args)
        .output()
        .map_err(|error| format!("failed to launch networksetup: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "networksetup command failed.".to_string()
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) fn parse_networksetup_mtu(text: &str) -> Option<u16> {
    // Examples:
    // "Active MTU: 1500 (Current Setting: 1500)"
    // "1500"
    for token in text.split(|c: char| !c.is_ascii_digit()) {
        if token.is_empty() {
            continue;
        }
        if let Ok(value) = token.parse::<u16>() {
            if (576..=9000).contains(&value) {
                return Some(value);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_networksetup_mtu;

    #[test]
    fn parses_get_mtu_output() {
        assert_eq!(
            parse_networksetup_mtu("Active MTU: 1500 (Current Setting: 1500)"),
            Some(1500)
        );
    }
}
