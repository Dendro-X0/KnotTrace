//! Shared link-speed / Wi‑Fi parsing (cfg-free for CI on all hosts).

#![allow(dead_code)]

use crate::types::WifiBand;

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WifiInterfaceSnapshot {
    pub name: String,
    pub ssid: Option<String>,
    pub signal_percent: Option<u8>,
    pub signal_dbm: Option<i16>,
    pub channel: Option<u32>,
    pub band: Option<WifiBand>,
    pub radio: Option<String>,
    pub receive_rate_mbps: Option<u32>,
}

pub fn band_from_channel(channel: u32) -> WifiBand {
    if (1..=14).contains(&channel) {
        WifiBand::Band24
    } else if (32..=177).contains(&channel) {
        WifiBand::Band5
    } else if channel > 0 {
        WifiBand::Band6
    } else {
        WifiBand::Unknown
    }
}

pub fn parse_signal_percent(raw: &str) -> Option<u8> {
    let digits: String = raw.chars().filter(|ch| ch.is_ascii_digit()).collect();
    let value: u8 = digits.parse().ok()?;
    Some(value.min(100))
}

pub fn parse_signal_dbm(raw: &str) -> Option<i16> {
    let trimmed = raw.trim();
    let mut num = String::new();
    let mut negative = false;
    for ch in trimmed.chars() {
        if ch == '-' && num.is_empty() {
            negative = true;
        } else if ch.is_ascii_digit() {
            num.push(ch);
        } else if !num.is_empty() {
            break;
        }
    }
    let value: i16 = num.parse().ok()?;
    Some(if negative { -value } else { value })
}

/// Parse `netsh wlan show interfaces` (Windows). Supports one or more interface blocks.
pub fn parse_netsh_wlan_interfaces(text: &str) -> Vec<WifiInterfaceSnapshot> {
    let mut out = Vec::new();
    let mut current = WifiInterfaceSnapshot::default();
    let mut has_name = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        if key == "name" {
            if has_name {
                out.push(std::mem::take(&mut current));
            }
            current.name = value.to_string();
            has_name = true;
            continue;
        }
        if !has_name {
            continue;
        }

        match key.as_str() {
            "ssid" => current.ssid = Some(value.to_string()),
            "signal" => current.signal_percent = parse_signal_percent(value),
            "channel" => {
                if let Ok(channel) = value
                    .chars()
                    .filter(|ch| ch.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                {
                    current.channel = Some(channel);
                    current.band = Some(band_from_channel(channel));
                }
            }
            "radio type" => current.radio = Some(value.to_string()),
            "receive rate (mbps)" | "receive rate" => {
                current.receive_rate_mbps = parse_speed_mbps(value);
            }
            _ => {}
        }
    }

    if has_name {
        out.push(current);
    }
    out
}

/// Parse a single `iw dev … link` dump.
pub fn parse_iw_link(text: &str) -> WifiInterfaceSnapshot {
    let mut snap = WifiInterfaceSnapshot::default();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(ssid) = trimmed.strip_prefix("SSID:") {
            snap.ssid = Some(ssid.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("signal:") {
            // e.g. "signal: -67 dBm"
            snap.signal_dbm = parse_signal_dbm(rest);
            if let Some(dbm) = snap.signal_dbm {
                // Rough percent map for UI when only dBm exists.
                let approx = ((dbm as i32 + 100) * 2).clamp(0, 100) as u8;
                snap.signal_percent = Some(approx);
            }
        } else if let Some(rest) = trimmed.strip_prefix("freq:") {
            if let Ok(freq) = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .parse::<u32>()
            {
                // MHz → coarse band
                snap.band = Some(if freq < 3000 {
                    WifiBand::Band24
                } else if freq < 6000 {
                    WifiBand::Band5
                } else {
                    WifiBand::Band6
                });
            }
        }
    }
    snap
}

pub fn wifi_signal_is_weak(percent: Option<u8>, dbm: Option<i16>) -> bool {
    if let Some(pct) = percent {
        if pct <= 40 {
            return true;
        }
    }
    if let Some(dbm) = dbm {
        if dbm <= -70 {
            return true;
        }
    }
    false
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

    #[test]
    fn parses_netsh_wlan_sample() {
        let sample = r#"
There is 1 interface on the system:

    Name                   : Wi-Fi
    State                  : connected
    SSID                   : HomeNet
    Radio type             : 802.11ax
    Channel                : 36
    Receive rate (Mbps)    : 1201
    Signal                 : 35%
"#;
        let parsed = parse_netsh_wlan_interfaces(sample);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "Wi-Fi");
        assert_eq!(parsed[0].ssid.as_deref(), Some("HomeNet"));
        assert_eq!(parsed[0].signal_percent, Some(35));
        assert_eq!(parsed[0].channel, Some(36));
        assert_eq!(parsed[0].band, Some(WifiBand::Band5));
        assert!(wifi_signal_is_weak(parsed[0].signal_percent, None));
    }
}
