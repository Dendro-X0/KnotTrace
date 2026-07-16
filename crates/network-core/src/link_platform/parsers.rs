//! Shared link-speed parsing (cfg-free for CI on all hosts).

#![allow(dead_code)]

/// Parse OS speed strings into Mbps.
/// Examples: `1 Gbps`, `100 Mbps`, `2.5 Gbps`, `1000`, `866.7 MBit/s`.
pub fn parse_speed_mbps(raw: &str) -> Option<u32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed.eq_ignore_ascii_case("unknown") {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    let numeric: String = lower
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    let value: f64 = numeric.parse().ok()?;
    if value <= 0.0 {
        return None;
    }

    let mbps = if lower.contains("gbit") || lower.contains("gbps") || lower.contains("gb/s") {
        value * 1000.0
    } else if lower.contains("kbit") || lower.contains("kbps") || lower.contains("kb/s") {
        value / 1000.0
    } else {
        // Mbps, MBit/s, bare number from sysfs, etc.
        value
    };

    Some(mbps.round().clamp(1.0, 1_000_000.0) as u32)
}

pub fn parse_duplex(raw: &str) -> Option<crate::types::LinkDuplex> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.is_empty() || lower == "unknown" || lower == "-" {
        return Some(crate::types::LinkDuplex::Unknown);
    }
    if lower.contains("half") {
        return Some(crate::types::LinkDuplex::Half);
    }
    if lower.contains("full") {
        return Some(crate::types::LinkDuplex::Full);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_speed_strings() {
        assert_eq!(parse_speed_mbps("1 Gbps"), Some(1000));
        assert_eq!(parse_speed_mbps("100 Mbps"), Some(100));
        assert_eq!(parse_speed_mbps("2.5 Gbps"), Some(2500));
        assert_eq!(parse_speed_mbps("866.7 MBit/s"), Some(867));
        assert_eq!(parse_speed_mbps("1000"), Some(1000));
        assert_eq!(parse_speed_mbps("-1"), None);
        assert_eq!(parse_speed_mbps("unknown"), None);
    }
}
