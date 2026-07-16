use crate::types::EnvironmentSnapshot;

pub fn platform_available() -> bool {
    false
}

pub fn platform_note() -> String {
    "MTU assist apply/restore is available on Windows, macOS, and Linux desktop builds.".to_string()
}

pub fn resolve_interface_name(environment: &EnvironmentSnapshot) -> Option<String> {
    environment
        .interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| environment.interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| iface.name.clone())
}

pub fn read_mtu(_interface_name: &str) -> Result<u16, String> {
    Err(platform_note())
}

pub fn set_mtu(_interface_name: &str, _mtu: u16) -> Result<(), String> {
    Err(platform_note())
}
