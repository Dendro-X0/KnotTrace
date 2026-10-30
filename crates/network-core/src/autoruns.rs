//! Band C4 — opt-in startup / autorun network-agent indicators.

use crate::types::*;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const AUTORUNS_CONSENT: &str = "knottrace.autoruns.consent.v1";

const NETWORK_KEYWORDS: &[&str] = &[
    "vpn",
    "proxy",
    "clash",
    "mihomo",
    "v2ray",
    "xray",
    "wireguard",
    "wg-quick",
    "openvpn",
    "softether",
    "shadowsocks",
    "ss-local",
    "sing-box",
    "singbox",
    "hysteria",
    "trojan",
    "tor",
    "outline",
    "warp",
    "tailscale",
    "zerotier",
    "nordvpn",
    "expressvpn",
    "mullvad",
    "protonvpn",
    "windscribe",
    "surfshark",
    "cloudflare",
    "nekoray",
    "v2rayn",
    "qv2ray",
];

#[derive(Debug, Error)]
pub enum AutorunError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid settings: {0}")]
    Invalid(String),
}

pub fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("autoruns_settings.json")
}

pub fn default_autorun_settings() -> AutorunSettings {
    AutorunSettings::default()
}

pub fn load_autorun_settings(data_dir: &Path) -> Result<AutorunSettings, AutorunError> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(default_autorun_settings());
    }
    let contents = std::fs::read_to_string(path)?;
    let settings: AutorunSettings = serde_json::from_str(&contents)?;
    Ok(normalize_autorun_settings(&settings))
}

pub fn save_autorun_settings(
    data_dir: &Path,
    settings: &AutorunSettings,
) -> Result<AutorunSettings, AutorunError> {
    if settings.enabled && !settings.consent_ack {
        return Err(AutorunError::Invalid(
            "Ack consent before enabling startup network-agent scan.".to_string(),
        ));
    }
    let normalized = normalize_autorun_settings(settings);
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        settings_path(data_dir),
        serde_json::to_string_pretty(&normalized)?,
    )?;
    Ok(normalized)
}

pub fn normalize_autorun_settings(settings: &AutorunSettings) -> AutorunSettings {
    if settings.enabled && !settings.consent_ack {
        return AutorunSettings {
            enabled: false,
            consent_ack: false,
        };
    }
    AutorunSettings {
        enabled: settings.enabled,
        consent_ack: settings.consent_ack,
    }
}

pub fn looks_network_related(name: &str, command: Option<&str>) -> bool {
    let hay = format!("{} {}", name, command.unwrap_or(""))
        .to_ascii_lowercase()
        .replace(['\\', '/', '-', '_', '.'], " ");
    NETWORK_KEYWORDS
        .iter()
        .any(|kw| hay.split_whitespace().any(|tok| tok.contains(kw)) || hay.contains(kw))
}

pub fn enrich_autoruns(report: &mut HealthReport, settings: &AutorunSettings) {
    if !settings.enabled || !settings.consent_ack {
        return;
    }
    if report.integrity.is_none() {
        return;
    }
    let snapshot = scan_autoruns();
    apply_autorun_snapshot(report, snapshot);
}

fn apply_autorun_snapshot(report: &mut HealthReport, snapshot: AutorunSnapshot) {
    let Some(board) = report.integrity.as_mut() else {
        return;
    };
    if snapshot.scanned && !snapshot.hits.is_empty() {
        let names: Vec<&str> = snapshot
            .hits
            .iter()
            .take(5)
            .map(|hit| hit.name.as_str())
            .collect();
        board.indicators.push(IntegrityIndicator {
            kind: IntegrityIndicatorKind::StartupNetworkAgent,
            severity: AlertLevel::Info,
            title: "Network-related startup entries detected".to_string(),
            message: format!(
                "Opt-in scan found startup/autorun entries that look network-related ({}). Confirm they match tools you installed — indicator only, not malware detection.",
                names.join(", ")
            ),
            evidence: snapshot
                .hits
                .iter()
                .take(8)
                .map(|hit| format!("{} @ {}", hit.name, hit.location))
                .collect::<Vec<_>>()
                .join("; "),
        });
        if matches!(
            board.level,
            IntegrityBoardLevel::None | IntegrityBoardLevel::Info
        ) {
            board.level = IntegrityBoardLevel::Info;
        }
        board.indicator_count = board.indicators.len().min(u8::MAX as usize) as u8;
    }
    board.autoruns = Some(snapshot);
}

fn scan_autoruns() -> AutorunSnapshot {
    #[cfg(windows)]
    {
        return scan_windows_run_key();
    }
    #[cfg(target_os = "linux")]
    {
        return scan_linux_autostart();
    }
    #[cfg(target_os = "macos")]
    {
        return scan_macos_launch_agents();
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        AutorunSnapshot {
            scanned: false,
            enabled: true,
            source: "autoruns_not_implemented".to_string(),
            hits: vec![],
            summary: "Startup scan is not implemented on this OS.".to_string(),
        }
    }
}

#[cfg(windows)]
fn scan_windows_run_key() -> AutorunSnapshot {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") else {
        return AutorunSnapshot {
            scanned: false,
            enabled: true,
            source: "hkcu_run".to_string(),
            hits: vec![],
            summary: "Could not open HKCU Run key.".to_string(),
        };
    };

    let mut hits = Vec::new();
    for entry in key.enum_values().flatten() {
        let (name, _) = entry;
        let Ok(command) = key.get_value::<String, _>(&name) else {
            continue;
        };
        if looks_network_related(&name, Some(&command)) {
            hits.push(AutorunHit {
                name,
                location: r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".to_string(),
                command_hint: Some(truncate_cmd(&command)),
            });
        }
        if hits.len() >= 12 {
            break;
        }
    }

    let summary = if hits.is_empty() {
        "Scanned HKCU Run; no network-related startup names matched.".to_string()
    } else {
        format!(
            "Found {} network-related HKCU Run entr{}.",
            hits.len(),
            if hits.len() == 1 { "y" } else { "ies" }
        )
    };

    AutorunSnapshot {
        scanned: true,
        enabled: true,
        source: "hkcu_run".to_string(),
        hits,
        summary,
    }
}

#[cfg(target_os = "linux")]
fn scan_linux_autostart() -> AutorunSnapshot {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return AutorunSnapshot {
            scanned: false,
            enabled: true,
            source: "xdg_autostart".to_string(),
            hits: vec![],
            summary: "HOME unset; cannot scan autostart.".to_string(),
        };
    };
    let dir = home.join(".config/autostart");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return AutorunSnapshot {
            scanned: true,
            enabled: true,
            source: "xdg_autostart".to_string(),
            hits: vec![],
            summary: "No ~/.config/autostart directory (or unreadable).".to_string(),
        };
    };

    let mut hits = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (name, exec) = parse_desktop_entry(&text);
        let display = name
            .clone()
            .unwrap_or_else(|| path.file_stem().and_then(|s| s.to_str()).unwrap_or("entry").to_string());
        if looks_network_related(&display, exec.as_deref())
            || looks_network_related(path.file_name().and_then(|s| s.to_str()).unwrap_or(""), exec.as_deref())
        {
            hits.push(AutorunHit {
                name: display,
                location: path.display().to_string(),
                command_hint: exec.map(|s| truncate_cmd(&s)),
            });
        }
        if hits.len() >= 12 {
            break;
        }
    }

    let summary = if hits.is_empty() {
        "Scanned XDG autostart; no network-related entries matched.".to_string()
    } else {
        format!("Found {} network-related XDG autostart entr{}.", hits.len(), if hits.len() == 1 { "y" } else { "ies" })
    };

    AutorunSnapshot {
        scanned: true,
        enabled: true,
        source: "xdg_autostart".to_string(),
        hits,
        summary,
    }
}

#[cfg(target_os = "macos")]
fn scan_macos_launch_agents() -> AutorunSnapshot {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return AutorunSnapshot {
            scanned: false,
            enabled: true,
            source: "launch_agents".to_string(),
            hits: vec![],
            summary: "HOME unset; cannot scan LaunchAgents.".to_string(),
        };
    };
    let dir = home.join("Library/LaunchAgents");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return AutorunSnapshot {
            scanned: true,
            enabled: true,
            source: "launch_agents".to_string(),
            hits: vec![],
            summary: "No ~/Library/LaunchAgents directory (or unreadable).".to_string(),
        };
    };

    let mut hits = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("plist") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("agent")
            .to_string();
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        if looks_network_related(&stem, Some(&text)) {
            hits.push(AutorunHit {
                name: stem,
                location: path.display().to_string(),
                command_hint: None,
            });
        }
        if hits.len() >= 12 {
            break;
        }
    }

    let summary = if hits.is_empty() {
        "Scanned LaunchAgents; no network-related names matched.".to_string()
    } else {
        format!(
            "Found {} network-related LaunchAgent{}.",
            hits.len(),
            if hits.len() == 1 { "" } else { "s" }
        )
    };

    AutorunSnapshot {
        scanned: true,
        enabled: true,
        source: "launch_agents".to_string(),
        hits,
        summary,
    }
}

fn truncate_cmd(cmd: &str) -> String {
    let trimmed = cmd.trim();
    if trimmed.len() <= 120 {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..117])
    }
}

/// Parse Name= / Exec= from a .desktop file (cfg-free for tests).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn parse_desktop_entry(text: &str) -> (Option<String>, Option<String>) {
    let mut name = None;
    let mut exec = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Name=") {
            if name.is_none() {
                name = Some(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("Exec=") {
            if exec.is_none() {
                exec = Some(rest.to_string());
            }
        }
    }
    (name, exec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_match_is_conservative() {
        assert!(looks_network_related("Clash for Windows", None));
        assert!(looks_network_related("app", Some("C:\\Tools\\wireguard.exe")));
        assert!(!looks_network_related("OneDrive", Some("C:\\Users\\a\\OneDrive.exe")));
        assert!(!looks_network_related("Steam", None));
    }

    #[test]
    fn settings_require_consent() {
        let dir = std::env::temp_dir().join(format!(
            "knottrace-autoruns-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let err = save_autorun_settings(
            &dir,
            &AutorunSettings {
                enabled: true,
                consent_ack: false,
            },
        );
        assert!(err.is_err());
        let ok = save_autorun_settings(
            &dir,
            &AutorunSettings {
                enabled: true,
                consent_ack: true,
            },
        )
        .unwrap();
        assert!(ok.enabled && ok.consent_ack);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_desktop_entry() {
        let text = "[Desktop Entry]\nName=Mihomo\nExec=/usr/bin/mihomo -d ~/.config/mihomo\n";
        let (name, exec) = parse_desktop_entry(text);
        assert_eq!(name.as_deref(), Some("Mihomo"));
        assert!(exec.unwrap().contains("mihomo"));
    }
}
