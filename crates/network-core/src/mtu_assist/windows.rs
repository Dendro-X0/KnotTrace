use crate::process::hidden_command;
use crate::types::{EnvironmentSnapshot, NetworkInterface};

pub fn platform_available() -> bool {
    true
}

pub fn platform_note() -> String {
    "Windows interface MTU clamp via netsh / PowerShell. Administrator approval may be required.".to_string()
}

pub fn resolve_interface_name(environment: &EnvironmentSnapshot) -> Option<String> {
    resolve_alias(&environment.interfaces)
}

pub fn read_mtu(interface_alias: &str) -> Result<u16, String> {
    let alias = escape_ps_single_quoted(interface_alias);
    let script = format!(
        "try {{ (Get-NetIPInterface -InterfaceAlias '{alias}' -AddressFamily IPv4 -ErrorAction Stop | Select-Object -First 1).NlMtu }} catch {{ '' }}"
    );
    if let Ok(output) = run_powershell(&script) {
        if let Ok(mtu) = output.trim().parse::<u16>() {
            return Ok(mtu);
        }
    }

    let output = hidden_command("netsh")
        .args(["interface", "ipv4", "show", "subinterfaces"])
        .output()
        .map_err(|error| format!("failed to launch netsh: {error}"))?;
    if !output.status.success() {
        return Err("Unable to read interface MTU.".to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_netsh_subinterface_mtu(&text, interface_alias)
        .ok_or_else(|| format!("MTU for `{interface_alias}` not found."))
}

pub fn set_mtu(interface_alias: &str, mtu: u16) -> Result<(), String> {
    let alias = escape_ps_single_quoted(interface_alias);
    let script = format!(
        "Set-NetIPInterface -InterfaceAlias '{alias}' -AddressFamily IPv4 -NlMtuBytes {mtu} -ErrorAction Stop"
    );
    if run_powershell(&script).is_ok() {
        return Ok(());
    }

    let output = hidden_command("netsh")
        .args([
            "interface",
            "ipv4",
            "set",
            "subinterface",
            interface_alias,
            &format!("mtu={mtu}"),
            "store=persistent",
        ])
        .output()
        .map_err(|error| format!("failed to launch netsh: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Failed to set interface MTU. Try running KnotTrace as administrator.".to_string()
        } else {
            stderr
        });
    }
    Ok(())
}

fn resolve_alias(interfaces: &[NetworkInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| {
            iface
                .friendly_name
                .clone()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| iface.name.clone())
        })
}

pub(crate) fn parse_netsh_subinterface_mtu(text: &str, alias: &str) -> Option<u16> {
    let alias_lower = alias.to_ascii_lowercase();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.to_ascii_lowercase().starts_with("mtu") {
            continue;
        }
        // Typical: "  1500       1     20  Ethernet"
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let name = parts[3..].join(" ");
        if name.eq_ignore_ascii_case(alias) || name.to_ascii_lowercase().contains(&alias_lower) {
            return parts[0].parse().ok();
        }
    }
    None
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
            "PowerShell command failed.".to_string()
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn escape_ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::parse_netsh_subinterface_mtu;

    #[test]
    fn parses_netsh_subinterfaces() {
        let text = r#"
MTU  MediaSenseState  Bytes In  Bytes Out  Interface
1500               1          0          0  Loopback Pseudo-Interface 1
1400               1       1200        800  Ethernet
"#;
        assert_eq!(parse_netsh_subinterface_mtu(text, "Ethernet"), Some(1400));
    }
}
