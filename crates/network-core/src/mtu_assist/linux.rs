use crate::process::hidden_command;
use crate::types::EnvironmentSnapshot;
use std::fs;
use std::path::Path;

pub fn platform_available() -> bool {
    true
}

pub fn platform_note() -> String {
    "Linux interface MTU via sysfs / ip link. Root or polkit approval may be required to apply.".to_string()
}

pub fn resolve_interface_name(environment: &EnvironmentSnapshot) -> Option<String> {
    environment
        .interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| environment.interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| iface.name.clone())
}

pub fn read_mtu(interface_name: &str) -> Result<u16, String> {
    let path = Path::new("/sys/class/net").join(interface_name).join("mtu");
    if path.exists() {
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        return text
            .trim()
            .parse()
            .map_err(|_| format!("Invalid MTU in {}", path.display()));
    }

    let output = hidden_command("ip")
        .args(["-o", "link", "show", "dev", interface_name])
        .output()
        .map_err(|error| format!("failed to launch ip: {error}"))?;
    if !output.status.success() {
        return Err(format!("Unable to read MTU for `{interface_name}`."));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_ip_link_mtu(&text).ok_or_else(|| format!("MTU for `{interface_name}` not found."))
}

pub fn set_mtu(interface_name: &str, mtu: u16) -> Result<(), String> {
    let output = hidden_command("ip")
        .args(["link", "set", "dev", interface_name, "mtu", &mtu.to_string()])
        .output()
        .map_err(|error| format!("failed to launch ip: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Failed to set interface MTU. Elevated privileges may be required.".to_string()
        } else {
            stderr
        });
    }
    Ok(())
}

pub(crate) fn parse_ip_link_mtu(text: &str) -> Option<u16> {
    // Example: "2: eth0: <BROADCAST,UP> mtu 1500 qdisc ..."
    let lower = text.to_ascii_lowercase();
    let idx = lower.find("mtu ")?;
    let rest = text[idx + 4..].trim_start();
    let token = rest.split_whitespace().next()?;
    token.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_ip_link_mtu;

    #[test]
    fn parses_ip_link_line() {
        let line = "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP";
        assert_eq!(parse_ip_link_mtu(line), Some(1500));
    }
}
