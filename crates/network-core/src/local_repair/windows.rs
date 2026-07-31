use crate::process::hidden_command;
use crate::types::NetworkInterface;

pub fn platform_available() -> bool {
    true
}

pub fn platform_note() -> String {
    "Windows local repair: flush DNS, renew DHCP, restart adapter. Administrator approval may be required.".to_string()
}

pub fn supports_dhcp_renew() -> bool {
    true
}

pub fn supports_adapter_restart() -> bool {
    true
}

pub fn flush_dns() -> Result<String, String> {
    let output = hidden_command("ipconfig")
        .args(["/flushdns"])
        .output()
        .map_err(|error| format!("failed to launch ipconfig: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "ipconfig /flushdns failed.".to_string()
        } else {
            stderr
        });
    }
    Ok("Flushed Windows DNS resolver cache.".to_string())
}

pub fn renew_dhcp(adapter_alias: &str) -> Result<String, String> {
    let release = hidden_command("ipconfig")
        .args(["/release"])
        .output()
        .map_err(|error| format!("failed to launch ipconfig /release: {error}"))?;
    if !release.status.success() {
        // Continue — some adapters reject release but accept renew.
    }
    let renew = hidden_command("ipconfig")
        .args(["/renew"])
        .output()
        .map_err(|error| format!("failed to launch ipconfig /renew: {error}"))?;
    if !renew.status.success() {
        let stderr = String::from_utf8_lossy(&renew.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!(
                "DHCP renew failed for `{adapter_alias}`. Try running KnotTrace as administrator."
            )
        } else {
            stderr
        });
    }
    Ok(format!(
        "Renewed DHCP lease (ipconfig /renew). Adapter context: `{adapter_alias}`."
    ))
}

pub fn restart_adapter(adapter_alias: &str) -> Result<String, String> {
    let alias = escape_ps_single_quoted(adapter_alias);
    let script = format!(
        "Disable-NetAdapter -Name '{alias}' -Confirm:$false -ErrorAction Stop; \
         Start-Sleep -Seconds 2; \
         Enable-NetAdapter -Name '{alias}' -Confirm:$false -ErrorAction Stop"
    );
    run_powershell(&script)?;
    Ok(format!(
        "Restarted adapter `{adapter_alias}` (disable/enable)."
    ))
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
            "PowerShell adapter command failed. Try running as administrator.".to_string()
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn escape_ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[allow(dead_code)]
pub fn resolve_from_interfaces(interfaces: &[NetworkInterface]) -> Option<String> {
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
