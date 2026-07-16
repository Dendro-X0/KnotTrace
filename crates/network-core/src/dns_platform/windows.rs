use crate::process::hidden_command;
use crate::types::NetworkInterface;

pub fn can_apply() -> bool {
    true
}

pub fn platform_note() -> String {
    "DNS changes apply to the active Windows network adapter. Administrator approval may be required.".to_string()
}

pub fn dns_target_for_interface(iface: &NetworkInterface) -> Option<String> {
    iface
        .friendly_name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .or_else(|| Some(iface.name.clone()))
}

pub fn read_dns_config(interface_alias: &str) -> Result<(Vec<String>, bool), String> {
    let alias = escape_ps_single_quoted(interface_alias);
    let script = format!(
        "$iface = Get-DnsClientServerAddress -InterfaceAlias '{alias}' -AddressFamily IPv4 -ErrorAction Stop; \
         $servers = @($iface.ServerAddresses | ForEach-Object {{ $_.ToString() }}); \
         $dhcp = (Get-NetIPInterface -InterfaceAlias '{alias}' -AddressFamily IPv4).Dhcp -eq 'Enabled'; \
         @{{ servers = $servers; dhcp = $dhcp }} | ConvertTo-Json -Compress"
    );

    let output = run_powershell(&script)?;
    let parsed: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| format!("invalid DNS config response: {error}"))?;

    let servers = parsed["servers"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let use_dhcp = parsed["dhcp"].as_bool().unwrap_or(servers.is_empty());
    Ok((servers, use_dhcp))
}

pub fn set_static_dns(interface_alias: &str, resolver: &str) -> Result<(), String> {
    let alias = escape_ps_single_quoted(interface_alias);
    let resolver = escape_ps_single_quoted(resolver);
    let script = format!(
        "Set-DnsClientServerAddress -InterfaceAlias '{alias}' -ServerAddresses ('{resolver}') -ErrorAction Stop"
    );
    run_powershell(&script).map(|_| ())
}

pub fn restore_dns_config(
    interface_alias: &str,
    previous_servers: &[String],
    use_dhcp: bool,
) -> Result<(), String> {
    let alias = escape_ps_single_quoted(interface_alias);
    if use_dhcp || previous_servers.is_empty() {
        let script = format!(
            "Set-DnsClientServerAddress -InterfaceAlias '{alias}' -ResetServerAddresses -ErrorAction Stop"
        );
        return run_powershell(&script).map(|_| ());
    }

    let servers = previous_servers
        .iter()
        .map(|server| format!("'{}'", escape_ps_single_quoted(server)))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        "Set-DnsClientServerAddress -InterfaceAlias '{alias}' -ServerAddresses @({servers}) -ErrorAction Stop"
    );
    run_powershell(&script).map(|_| ())
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
            "PowerShell command failed. Try running Network Companion as administrator.".to_string()
        } else {
            detail
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn escape_ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}
