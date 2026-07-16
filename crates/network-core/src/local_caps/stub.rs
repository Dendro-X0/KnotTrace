use crate::types::NetworkInterface;

pub fn platform_available() -> bool {
    false
}

pub fn platform_note() -> String {
    "Windows local caps (TCP auto-tuning / NIC power) are only available on Windows.".to_string()
}

pub fn read_autotuning_level() -> Result<String, String> {
    Err(platform_note())
}

pub fn set_autotuning_level(_level: &str) -> Result<(), String> {
    Err(platform_note())
}

pub fn read_adapter_power_saving(_adapter_alias: &str) -> Result<Option<bool>, String> {
    Err(platform_note())
}

pub fn set_adapter_power_saving(_adapter_alias: &str, _allow_turn_off: bool) -> Result<(), String> {
    Err(platform_note())
}

pub fn resolve_adapter_alias(interfaces: &[NetworkInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|iface| iface.is_default_route)
        .or_else(|| interfaces.iter().find(|iface| iface.is_up))
        .map(|iface| iface.name.clone())
}
