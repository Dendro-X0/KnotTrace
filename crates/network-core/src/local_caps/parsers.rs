//! Shared parsing for Windows local-caps probes (cfg-free for CI).

#![allow(dead_code)]

/// Normalize TCP auto-tuning level labels from PowerShell / netsh.
pub fn normalize_autotuning_level(raw: &str) -> String {
    let compact = raw.trim().to_ascii_lowercase().replace([' ', '-', '_'], "");
    match compact.as_str() {
        "normal" => "normal".to_string(),
        "disabled" => "disabled".to_string(),
        "restricted" => "restricted".to_string(),
        "highlyrestricted" => "highlyrestricted".to_string(),
        "experimental" => "experimental".to_string(),
        other if other.is_empty() => "unknown".to_string(),
        other => other.to_string(),
    }
}

pub fn autotuning_is_ok(level: &str) -> bool {
    matches!(
        normalize_autotuning_level(level).as_str(),
        "normal" | "experimental"
    )
}

pub fn parse_allow_computer_turn_off(raw: &str) -> Option<bool> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.is_empty() || lower == "unsupported" || lower == "notsupported" {
        return None;
    }
    if matches!(lower.as_str(), "enabled" | "true" | "yes" | "on" | "1") {
        return Some(true);
    }
    if matches!(lower.as_str(), "disabled" | "false" | "no" | "off" | "0") {
        return Some(false);
    }
    None
}

/// Parse `netsh interface tcp show global` for Auto-Tuning Level.
pub fn parse_netsh_autotuning(output: &str) -> Option<String> {
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("auto-tuning level") || lower.contains("autotuning level") {
            if let Some((_, rhs)) = line.split_once(':') {
                let level = normalize_autotuning_level(rhs);
                if level != "unknown" {
                    return Some(level);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_autotuning_levels() {
        assert_eq!(normalize_autotuning_level("Normal"), "normal");
        assert_eq!(normalize_autotuning_level("highly restricted"), "highlyrestricted");
        assert!(autotuning_is_ok("normal"));
        assert!(!autotuning_is_ok("disabled"));
        assert!(!autotuning_is_ok("HighlyRestricted"));
    }

    #[test]
    fn parses_netsh_global() {
        let sample = "\
Querying active state...

TCP Global Parameters
----------------------------------------------
Receive-Side Scaling State          : enabled
Receive Window Auto-Tuning Level    : highlyrestricted
Add-On Congestion Control Provider  : default
";
        assert_eq!(
            parse_netsh_autotuning(sample).as_deref(),
            Some("highlyrestricted")
        );
    }

    #[test]
    fn parses_power_flag() {
        assert_eq!(parse_allow_computer_turn_off("Enabled"), Some(true));
        assert_eq!(parse_allow_computer_turn_off("Disabled"), Some(false));
        assert_eq!(parse_allow_computer_turn_off("NotSupported"), None);
    }
}
