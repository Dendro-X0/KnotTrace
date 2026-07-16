use crate::process::hidden_command;
use crate::types::NetworkInterface;
use super::parsers::{parse_nmcli_dns_list, parse_resolvectl_dns};
use std::process::Command;

pub fn can_apply() -> bool {
    command_exists("nmcli") || command_exists("resolvectl")
}

pub fn platform_note() -> String {
    "DNS changes apply to the active Linux interface via NetworkManager (nmcli) or systemd-resolved. Polkit/administrator approval may be required.".to_string()
}

pub fn dns_target_for_interface(iface: &NetworkInterface) -> Option<String> {
    // nmcli / resolvectl operate on kernel device names (wlan0, eth0, enp0s3).
    Some(iface.name.clone())
}

pub fn read_dns_config(interface_alias: &str) -> Result<(Vec<String>, bool), String> {
    if command_exists("nmcli") {
        read_via_nmcli(interface_alias)
    } else if command_exists("resolvectl") {
        read_via_resolvectl(interface_alias)
    } else {
        Err(platform_note())
    }
}

pub fn set_static_dns(interface_alias: &str, resolver: &str) -> Result<(), String> {
    if command_exists("nmcli") {
        set_via_nmcli(interface_alias, resolver)
    } else if command_exists("resolvectl") {
        run_checked(
            "resolvectl",
            &["dns", interface_alias, resolver],
            "Failed to set DNS via resolvectl",
        )
        .map(|_| ())
    } else {
        Err(platform_note())
    }
}

pub fn restore_dns_config(
    interface_alias: &str,
    previous_servers: &[String],
    use_dhcp: bool,
) -> Result<(), String> {
    if command_exists("nmcli") {
        restore_via_nmcli(interface_alias, previous_servers, use_dhcp)
    } else if command_exists("resolvectl") {
        if use_dhcp || previous_servers.is_empty() {
            run_checked(
                "resolvectl",
                &["revert", interface_alias],
                "Failed to revert DNS via resolvectl",
            )
            .map(|_| ())
        } else {
            let mut args = vec!["dns", interface_alias];
            let servers: Vec<&str> = previous_servers.iter().map(String::as_str).collect();
            args.extend(servers);
            run_checked("resolvectl", &args, "Failed to restore DNS via resolvectl").map(|_| ())
        }
    } else {
        Err(platform_note())
    }
}

fn read_via_nmcli(interface_alias: &str) -> Result<(Vec<String>, bool), String> {
    let dns_out = run_checked(
        "nmcli",
        &["-g", "IP4.DNS", "device", "show", interface_alias],
        "Failed to read DNS via nmcli",
    )?;
    let servers = parse_nmcli_dns_list(&dns_out);

    let ignore_auto = connection_ignore_auto_dns(interface_alias).unwrap_or(false);
    let use_dhcp = !ignore_auto;
    Ok((servers, use_dhcp))
}

fn connection_ignore_auto_dns(interface_alias: &str) -> Result<bool, String> {
    let connection = run_checked(
        "nmcli",
        &["-g", "GENERAL.CONNECTION", "device", "show", interface_alias],
        "Failed to resolve NetworkManager connection",
    )?;
    let connection = connection.trim();
    if connection.is_empty() || connection == "--" {
        return Ok(false);
    }

    let value = run_checked(
        "nmcli",
        &["-g", "ipv4.ignore-auto-dns", "connection", "show", connection],
        "Failed to read ipv4.ignore-auto-dns",
    )?;
    Ok(value.trim().eq_ignore_ascii_case("yes"))
}

fn set_via_nmcli(interface_alias: &str, resolver: &str) -> Result<(), String> {
    run_checked(
        "nmcli",
        &[
            "device",
            "modify",
            interface_alias,
            "ipv4.dns",
            resolver,
            "ipv4.ignore-auto-dns",
            "yes",
        ],
        "Failed to set DNS via nmcli",
    )
    .map(|_| ())
}

fn restore_via_nmcli(
    interface_alias: &str,
    previous_servers: &[String],
    use_dhcp: bool,
) -> Result<(), String> {
    if use_dhcp || previous_servers.is_empty() {
        run_checked(
            "nmcli",
            &[
                "device",
                "modify",
                interface_alias,
                "ipv4.dns",
                "",
                "ipv4.ignore-auto-dns",
                "no",
            ],
            "Failed to restore DHCP DNS via nmcli",
        )
        .map(|_| ())
    } else {
        let joined = previous_servers.join(" ");
        run_checked(
            "nmcli",
            &[
                "device",
                "modify",
                interface_alias,
                "ipv4.dns",
                &joined,
                "ipv4.ignore-auto-dns",
                "yes",
            ],
            "Failed to restore static DNS via nmcli",
        )
        .map(|_| ())
    }
}

fn read_via_resolvectl(interface_alias: &str) -> Result<(Vec<String>, bool), String> {
    let output = run_checked(
        "resolvectl",
        &["dns", interface_alias],
        "Failed to read DNS via resolvectl",
    )?;
    let servers = parse_resolvectl_dns(&output);
    // Without NM metadata we treat pre-apply state as revertible (DHCP/link defaults).
    Ok((servers, true))
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_checked(program: &str, args: &[&str], context: &str) -> Result<String, String> {
    let output = hidden_command(program)
        .args(args)
        .output()
        .map_err(|error| format!("{context}: failed to launch {program}: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(if detail.is_empty() {
            format!("{context}. You may need to approve a polkit prompt or run with sufficient privileges.")
        } else {
            format!("{context}: {detail}")
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
