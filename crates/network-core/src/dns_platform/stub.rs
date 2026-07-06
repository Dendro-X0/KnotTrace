pub fn can_apply() -> bool {
    false
}

pub fn platform_note() -> String {
    "DNS assist is not available on this platform yet.".to_string()
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
