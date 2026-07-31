use crate::process::hidden_command;

pub fn platform_available() -> bool {
    true
}

pub fn platform_note() -> String {
    "Linux local repair: flush resolver caches and renew DHCP (nmcli/dhclient). Root/polkit may be required.".to_string()
}

pub fn supports_dhcp_renew() -> bool {
    true
}

pub fn supports_adapter_restart() -> bool {
    true
}

pub fn flush_dns() -> Result<String, String> {
    // Best-effort across common stacks.
    let mut notes = Vec::new();
    if let Ok(output) = hidden_command("resolvectl").args(["flush-caches"]).output() {
        if output.status.success() {
            notes.push("resolvectl flush-caches");
        }
    }
    if let Ok(output) = hidden_command("systemd-resolve")
        .args(["--flush-caches"])
        .output()
    {
        if output.status.success() {
            notes.push("systemd-resolve --flush-caches");
        }
    }
    if notes.is_empty() {
        return Err(
            "No supported DNS flush tool found (tried resolvectl / systemd-resolve).".to_string(),
        );
    }
    Ok(format!("Flushed DNS cache via {}.", notes.join(", ")))
}

pub fn renew_dhcp(adapter_alias: &str) -> Result<String, String> {
    if let Ok(output) = hidden_command("nmcli")
        .args(["device", "reapply", adapter_alias])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Reapplied NetworkManager config for `{adapter_alias}`."));
        }
    }
    let output = hidden_command("dhclient")
        .args(["-r", adapter_alias])
        .output();
    let _ = output;
    let renew = hidden_command("dhclient")
        .args([adapter_alias])
        .output()
        .map_err(|error| format!("failed to launch dhclient: {error}"))?;
    if !renew.status.success() {
        return Err(format!(
            "DHCP renew failed for `{adapter_alias}` (nmcli/dhclient)."
        ));
    }
    Ok(format!("Renewed DHCP via dhclient on `{adapter_alias}`."))
}

pub fn restart_adapter(adapter_alias: &str) -> Result<String, String> {
    if let Ok(output) = hidden_command("nmcli")
        .args(["device", "disconnect", adapter_alias])
        .output()
    {
        if output.status.success() {
            let _ = hidden_command("nmcli")
                .args(["device", "connect", adapter_alias])
                .output();
            return Ok(format!(
                "Cycled NetworkManager device `{adapter_alias}`."
            ));
        }
    }
    let down = hidden_command("ip")
        .args(["link", "set", adapter_alias, "down"])
        .output()
        .map_err(|error| format!("failed to launch ip: {error}"))?;
    if !down.status.success() {
        return Err(format!("Could not bring `{adapter_alias}` down."));
    }
    let up = hidden_command("ip")
        .args(["link", "set", adapter_alias, "up"])
        .output()
        .map_err(|error| format!("failed to launch ip: {error}"))?;
    if !up.status.success() {
        return Err(format!("Could not bring `{adapter_alias}` up."));
    }
    Ok(format!("Restarted link `{adapter_alias}` via ip link."))
}
