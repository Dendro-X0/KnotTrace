use crate::types::NetworkInterface;

pub fn can_apply() -> bool {
    false
}

pub fn platform_note() -> String {
    "DNS assist apply/restore is not available on this platform.".to_string()
}

pub fn dns_target_for_interface(iface: &NetworkInterface) -> Option<String> {
    iface
        .friendly_name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .or_else(|| Some(iface.name.clone()))
}

pub fn read_dns_config(_interface_alias: &str) -> Result<(Vec<String>, bool), String> {
    Err(platform_note())
}

pub fn set_static_dns(_interface_alias: &str, _resolver: &str) -> Result<(), String> {
    Err(platform_note())
}

pub fn restore_dns_config(
    _interface_alias: &str,
    _previous_servers: &[String],
    _use_dhcp: bool,
) -> Result<(), String> {
    Err(platform_note())
}
