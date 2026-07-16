use crate::process::hidden_command;
use crate::types::NetworkInterface;
use super::parsers::{
    normalize_autotuning_level, parse_allow_computer_turn_off, parse_netsh_autotuning,
};

pub fn platform_available() -> bool {
    true
}

pub fn platform_note() -> String {
    "Windows TCP auto-tuning and NIC power settings. Administrator approval may be required to repair.".to_string()
}

pub fn read_autotuning_level() -> Result<String, String> {
    // Prefer Get-NetTCPSetting; fall back to netsh parsing.
    let script = "try { (Get-NetTCPSetting -SettingName Internet -ErrorAction Stop).AutoTuningLevelLocal.ToString() } catch { '' }";
    let output = run_powershell(script)?;
    let trimmed = output.trim();
    if !trimmed.is_empty() {
        return Ok(normalize_autotuning_level(trimmed));
    }

    let netsh = hidden_command("netsh")
        .args(["interface", "tcp", "show", "global"])
        .output()
        .map_err(|error| format!("failed to launch netsh: {error}"))?;
    if !netsh.status.success() {
        return Err("Unable to read TCP auto-tuning level.".to_string());
    }
    let text = String::from_utf8_lossy(&netsh.stdout);
    parse_netsh_autotuning(&text).ok_or_else(|| "TCP auto-tuning level not found.".to_string())
}

pub fn set_autotuning_level(level: &str) -> Result<(), String> {
    let normalized = normalize_autotuning_level(level);
    let ps_level = match normalized.as_str() {
        "normal" => "Normal",
        "disabled" => "Disabled",
        "restricted" => "Restricted",
        "highlyrestricted" => "HighlyRestricted",
        "experimental" => "Experimental",
        other => {
            return Err(format!("Unsupported auto-tuning level: {other}"));
        }
    };

    let script = format!(
        "Set-NetTCPSetting -SettingName Internet -AutoTuningLevelLocal {ps_level} -ErrorAction Stop"
    );
    if run_powershell(&script).is_ok() {
        return Ok(());
    }

    // Fallback for older hosts.
    let netsh_level = match normalized.as_str() {
        "highlyrestricted" => "highlyrestricted",
        other => other,
    };
    let output = hidden_command("netsh")
        .args([
            "interface",
            "tcp",
            "set",
            "global",
            &format!("autotuninglevel={netsh_level}"),
        ])
        .output()
        .map_err(|error| format!("failed to launch netsh: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Failed to set TCP auto-tuning. Try running KnotTrace as administrator.".to_string()
        } else {
            stderr
        });
    }
    Ok(())
}

pub fn read_adapter_power_saving(adapter_alias: &str) -> Result<Option<bool>, String> {
    let alias = escape_ps_single_quoted(adapter_alias);
    let script = format!(
        "try {{ $pm = Get-NetAdapterPowerManagement -Name '{alias}' -ErrorAction Stop; \
         if ($null -eq $pm.AllowComputerToTurnOffDevice) {{ '' }} else {{ $pm.AllowComputerToTurnOffDevice.ToString() }} }} catch {{ '' }}"
    );
    let output = run_powershell(&script)?;
    Ok(parse_allow_computer_turn_off(output.trim()))
}

pub fn set_adapter_power_saving(adapter_alias: &str, allow_turn_off: bool) -> Result<(), String> {
    let alias = escape_ps_single_quoted(adapter_alias);
    let value = if allow_turn_off { "Enabled" } else { "Disabled" };
    let script = format!(
        "Set-NetAdapterPowerManagement -Name '{alias}' -AllowComputerToTurnOffDevice {value} -ErrorAction Stop"
    );
    run_powershell(&script).map(|_| ())
}

pub fn resolve_adapter_alias(interfaces: &[NetworkInterface]) -> Option<String> {
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
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(if detail.is_empty() {
            "PowerShell command failed. Try running KnotTrace as administrator.".to_string()
        } else {
            detail
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn escape_ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}
