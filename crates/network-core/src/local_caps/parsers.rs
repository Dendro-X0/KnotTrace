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

/// Classify Linux `/proc` TCP knobs into the shared auto-tuning level vocabulary.
pub fn classify_linux_tcp_caps(window_scaling: &str, tcp_rmem: &str) -> String {
    let scaling = window_scaling.trim();
    if scaling == "0" {
        return "disabled".to_string();
    }
    let max_rmem = tcp_rmem
        .split_whitespace()
        .nth(2)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(u64::MAX);
    // Below ~1 MiB max receive buffer is a practical high-BDP cap.
    if max_rmem > 0 && max_rmem < 1_048_576 {
        return "restricted".to_string();
    }
    "normal".to_string()
}

/// macOS `doautorcvbuf` / `doautosndbuf` (1 = enabled).
pub fn classify_macos_tcp_caps(do_autorcvbuf: &str, do_autosndbuf: &str) -> String {
    let rcv = do_autorcvbuf.trim();
    let snd = do_autosndbuf.trim();
    if rcv == "0" || snd == "0" {
        return "disabled".to_string();
    }
    "normal".to_string()
}

/// Linux sysfs `power/control`: `auto` means runtime PM may power the device down.
pub fn parse_sys_power_control(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(true),
        "on" => Some(false),
        _ => None,
    }
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
    fn classify_linux_and_macos_caps() {
        assert_eq!(classify_linux_tcp_caps("0", "4096 87380 6291456"), "disabled");
        assert_eq!(classify_linux_tcp_caps("1", "4096 87380 65535"), "restricted");
        assert_eq!(classify_linux_tcp_caps("1", "4096 87380 6291456"), "normal");
        assert_eq!(classify_macos_tcp_caps("1", "1"), "normal");
        assert_eq!(classify_macos_tcp_caps("0", "1"), "disabled");
        assert_eq!(parse_sys_power_control("auto\n"), Some(true));
        assert_eq!(parse_sys_power_control("on"), Some(false));
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
