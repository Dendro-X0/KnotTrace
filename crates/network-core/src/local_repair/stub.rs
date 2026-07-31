pub fn platform_available() -> bool {
    false
}

pub fn platform_note() -> String {
    "Local repair actions are available on Windows, macOS, and Linux desktop builds.".to_string()
}

pub fn supports_dhcp_renew() -> bool {
    false
}

pub fn supports_adapter_restart() -> bool {
    false
}

pub fn flush_dns() -> Result<String, String> {
    Err(platform_note())
}

pub fn renew_dhcp(_adapter_alias: &str) -> Result<String, String> {
    Err(platform_note())
}

pub fn restart_adapter(_adapter_alias: &str) -> Result<String, String> {
    Err(platform_note())
}
