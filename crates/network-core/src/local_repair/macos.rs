use crate::process::hidden_command;

pub fn platform_available() -> bool {
    true
}

pub fn platform_note() -> String {
    "macOS local repair: flush DNS and renew DHCP via networksetup/dscacheutil. Admin may be required.".to_string()
}

pub fn supports_dhcp_renew() -> bool {
    true
}

pub fn supports_adapter_restart() -> bool {
    false
}

pub fn flush_dns() -> Result<String, String> {
    let output = hidden_command("dscacheutil")
        .args(["-flushcache"])
        .output()
        .map_err(|error| format!("failed to launch dscacheutil: {error}"))?;
    let _ = hidden_command("killall")
        .args(["-HUP", "mDNSResponder"])
        .output();
    if !output.status.success() {
        return Err("dscacheutil -flushcache failed.".to_string());
    }
    Ok("Flushed macOS DNS cache (dscacheutil + mDNSResponder HUP).".to_string())
}

pub fn renew_dhcp(adapter_alias: &str) -> Result<String, String> {
    // networksetup -setdhcp <service>
    let output = hidden_command("/usr/sbin/networksetup")
        .args(["-setdhcp", adapter_alias])
        .output()
        .map_err(|error| format!("failed to launch networksetup: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("DHCP renew failed for `{adapter_alias}`.")
        } else {
            stderr
        });
    }
    Ok(format!("Requested DHCP renew for service `{adapter_alias}`."))
}

pub fn restart_adapter(_adapter_alias: &str) -> Result<String, String> {
    Err("Adapter restart is not automated on macOS in this version.".to_string())
}
