//! Opt-in suspicious-activity indicators (script-host persistence / mining-like processes).
//! Indicators only — never malware verdicts, kill, or quarantine.

use crate::types::*;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const SUSPICIOUS_ACTIVITY_CONSENT: &str = "knottrace.suspicious_activity.consent.v1";

const SCRIPT_HOSTS: &[&str] = &[
    "powershell",
    "powershell.exe",
    "pwsh",
    "pwsh.exe",
    "wscript",
    "wscript.exe",
    "cscript",
    "cscript.exe",
    "mshta",
    "mshta.exe",
];

const SCRIPT_EXTS: &[&str] = &["ps1", "vbs", "js", "jse", "wsf", "bat", "cmd"];

const MINING_KEYWORDS: &[&str] = &[
    "xmrig",
    "ethminer",
    "minerd",
    "nicehash",
    "stratum",
    "hashrate",
    "cryptonight",
    "randomx",
    "nbminer",
    "phoenixminer",
    "gminer",
    "t-rex",
    "trexminer",
    "lolminer",
    "cgminer",
    "bfgminer",
];

const RISKY_PATH_TOKENS: &[&str] = &[
    "\\temp\\",
    "/temp/",
    "\\tmp\\",
    "/tmp/",
    "\\downloads\\",
    "/downloads/",
    "\\appdata\\local\\temp\\",
    "/appdata/local/temp/",
];

#[derive(Debug, Error)]
pub enum SuspiciousActivityError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid settings: {0}")]
    Invalid(String),
}

pub fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("suspicious_activity_settings.json")
}

pub fn default_suspicious_activity_settings() -> SuspiciousActivitySettings {
    SuspiciousActivitySettings::default()
}

pub fn load_suspicious_activity_settings(
    data_dir: &Path,
) -> Result<SuspiciousActivitySettings, SuspiciousActivityError> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(default_suspicious_activity_settings());
    }
    let contents = std::fs::read_to_string(path)?;
    let settings: SuspiciousActivitySettings = serde_json::from_str(&contents)?;
    Ok(normalize_suspicious_activity_settings(&settings))
}

pub fn save_suspicious_activity_settings(
    data_dir: &Path,
    settings: &SuspiciousActivitySettings,
) -> Result<SuspiciousActivitySettings, SuspiciousActivityError> {
    if settings.enabled && !settings.consent_ack {
        return Err(SuspiciousActivityError::Invalid(
            "Ack consent before enabling suspicious-activity indicators.".to_string(),
        ));
    }
    let normalized = normalize_suspicious_activity_settings(settings);
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        settings_path(data_dir),
        serde_json::to_string_pretty(&normalized)?,
    )?;
    Ok(normalized)
}

pub fn normalize_suspicious_activity_settings(
    settings: &SuspiciousActivitySettings,
) -> SuspiciousActivitySettings {
    if settings.enabled && !settings.consent_ack {
        return SuspiciousActivitySettings {
            enabled: false,
            consent_ack: false,
        };
    }
    SuspiciousActivitySettings {
        enabled: settings.enabled,
        consent_ack: settings.consent_ack,
    }
}

/// True when a startup command looks like a script host launch.
pub fn looks_script_host_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    if SCRIPT_HOSTS.iter().any(|host| token_contains(&lower, host)) {
        return true;
    }
    // cmd /c … powershell …
    if lower.contains("cmd")
        && (lower.contains("/c") || lower.contains("/k"))
        && SCRIPT_HOSTS.iter().any(|host| lower.contains(host))
    {
        return true;
    }
    false
}

pub fn looks_elevated_script_risk(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    lower.contains("-enc")
        || lower.contains("-encodedcommand")
        || lower.contains("downloadstring")
        || lower.contains("http://")
        || lower.contains("https://")
        || RISKY_PATH_TOKENS.iter().any(|tok| lower.contains(tok))
}

pub fn looks_mining_like(name: &str, path: Option<&str>, cmdline: Option<&str>) -> bool {
    let hay = format!(
        "{} {} {}",
        name,
        path.unwrap_or(""),
        cmdline.unwrap_or("")
    )
    .to_ascii_lowercase();
    if MINING_KEYWORDS.iter().any(|kw| hay.contains(kw)) {
        return true;
    }
    let in_risky_path = path
        .map(|p| {
            let lower = p.to_ascii_lowercase().replace('/', "\\");
            RISKY_PATH_TOKENS.iter().any(|tok| lower.contains(tok.trim_matches('/')))
                || lower.contains("\\temp\\")
                || lower.contains("\\downloads\\")
                || lower.contains("/tmp/")
                || lower.contains("/downloads/")
        })
        .unwrap_or(false);
    if in_risky_path {
        let cmd = cmdline.unwrap_or("").to_ascii_lowercase();
        return cmd.contains("pool")
            || cmd.contains("stratum")
            || cmd.contains("wallet")
            || cmd.contains("--algo")
            || cmd.contains("-o stratum");
    }
    false
}

fn token_contains(hay: &str, needle: &str) -> bool {
    hay.split(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_')
        .any(|tok| tok == needle || tok.ends_with(needle))
        || hay.contains(needle)
}

pub fn enrich_suspicious_activity(report: &mut HealthReport, settings: &SuspiciousActivitySettings) {
    if !settings.enabled || !settings.consent_ack {
        return;
    }
    if report.integrity.is_none() {
        return;
    }
    let snapshot = scan_suspicious_activity();
    apply_suspicious_activity_snapshot(report, snapshot);
}

fn apply_suspicious_activity_snapshot(report: &mut HealthReport, snapshot: SuspiciousActivitySnapshot) {
    let Some(board) = report.integrity.as_mut() else {
        return;
    };

    if snapshot.scanned && !snapshot.persistence_hits.is_empty() {
        let names: Vec<&str> = snapshot
            .persistence_hits
            .iter()
            .take(5)
            .map(|hit| hit.name.as_str())
            .collect();
        board.indicators.push(IntegrityIndicator {
            kind: IntegrityIndicatorKind::ScriptHostPersistence,
            severity: AlertLevel::Warning,
            title: "Script-host persistence indicators".to_string(),
            message: format!(
                "Opt-in scan found startup entries that launch script hosts ({}). Indicator only — not malware detection; confirm you installed them.",
                names.join(", ")
            ),
            evidence: snapshot
                .persistence_hits
                .iter()
                .take(8)
                .map(|hit| {
                    format!(
                        "{} @ {}{}",
                        hit.name,
                        hit.location,
                        hit.detail
                            .as_ref()
                            .map(|d| format!(" ({d})"))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
        });
        bump_board_level(board, IntegrityBoardLevel::Caution);
    }

    if snapshot.scanned && !snapshot.mining_hits.is_empty() {
        let names: Vec<&str> = snapshot
            .mining_hits
            .iter()
            .take(5)
            .map(|hit| hit.name.as_str())
            .collect();
        board.indicators.push(IntegrityIndicator {
            kind: IntegrityIndicatorKind::MiningLikeProcess,
            severity: AlertLevel::Warning,
            title: "Mining-like process indicators".to_string(),
            message: format!(
                "Opt-in scan found processes whose names/paths/command lines look mining-related ({}). Indicator only — not malware detection.",
                names.join(", ")
            ),
            evidence: snapshot
                .mining_hits
                .iter()
                .take(8)
                .map(|hit| {
                    format!(
                        "{} @ {}{}",
                        hit.name,
                        hit.location,
                        hit.detail
                            .as_ref()
                            .map(|d| format!(" ({d})"))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
        });
        bump_board_level(board, IntegrityBoardLevel::Caution);
    }

    board.indicator_count = board.indicators.len().min(u8::MAX as usize) as u8;
    board
        .not_checked
        .retain(|s| s != "Suspicious activity heuristics (script-host / mining-like)");
    board.suspicious_activity = Some(snapshot);
}

fn bump_board_level(board: &mut IntegrityBoardReport, at_least: IntegrityBoardLevel) {
    let rank = |level: IntegrityBoardLevel| match level {
        IntegrityBoardLevel::None => 0,
        IntegrityBoardLevel::Info => 1,
        IntegrityBoardLevel::Caution => 2,
        IntegrityBoardLevel::Elevated => 3,
    };
    if rank(board.level) < rank(at_least) {
        board.level = at_least;
    }
}

fn scan_suspicious_activity() -> SuspiciousActivitySnapshot {
    let mut persistence_hits = scan_persistence_hits();
    let mut mining_hits = scan_mining_hits();
    persistence_hits.truncate(8);
    mining_hits.truncate(8);

    let source = persistence_source_label();
    let summary = match (persistence_hits.is_empty(), mining_hits.is_empty()) {
        (true, true) => {
            "Scanned persistence + processes; no script-host or mining-like indicators matched."
                .to_string()
        }
        (false, true) => format!(
            "Found {} script-host persistence indicator(s).",
            persistence_hits.len()
        ),
        (true, false) => format!(
            "Found {} mining-like process indicator(s).",
            mining_hits.len()
        ),
        (false, false) => format!(
            "Found {} persistence and {} mining-like indicator(s).",
            persistence_hits.len(),
            mining_hits.len()
        ),
    };

    SuspiciousActivitySnapshot {
        scanned: true,
        enabled: true,
        source,
        persistence_hits,
        mining_hits,
        summary,
    }
}

fn persistence_source_label() -> String {
    #[cfg(windows)]
    {
        return "hkcu_run+startup_folder".to_string();
    }
    #[cfg(target_os = "linux")]
    {
        return "xdg_autostart+ps".to_string();
    }
    #[cfg(target_os = "macos")]
    {
        return "launch_agents+ps".to_string();
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        "suspicious_activity_partial".to_string()
    }
}

fn scan_persistence_hits() -> Vec<SuspiciousActivityHit> {
    #[cfg(windows)]
    {
        return scan_windows_persistence();
    }
    #[cfg(target_os = "linux")]
    {
        return scan_linux_persistence();
    }
    #[cfg(target_os = "macos")]
    {
        return scan_macos_persistence();
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

fn scan_mining_hits() -> Vec<SuspiciousActivityHit> {
    #[cfg(windows)]
    {
        return scan_windows_mining_processes();
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        return scan_unix_mining_processes();
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

#[cfg(windows)]
fn scan_windows_persistence() -> Vec<SuspiciousActivityHit> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let mut hits = Vec::new();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        for entry in key.enum_values().flatten() {
            let (name, _) = entry;
            let Ok(command) = key.get_value::<String, _>(&name) else {
                continue;
            };
            if looks_script_host_command(&command) {
                let risk = if looks_elevated_script_risk(&command) {
                    "elevated risk tokens"
                } else {
                    "script host"
                };
                hits.push(SuspiciousActivityHit {
                    name,
                    location: r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".to_string(),
                    detail: Some(format!("{risk}: {}", truncate_cmd(&command))),
                });
            }
            if hits.len() >= 8 {
                return hits;
            }
        }
    }

    if let Some(startup) = windows_startup_dir() {
        if let Ok(entries) = std::fs::read_dir(&startup) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if !SCRIPT_EXTS.iter().any(|e| *e == ext.as_str()) {
                    continue;
                }
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("script")
                    .to_string();
                hits.push(SuspiciousActivityHit {
                    name,
                    location: path.display().to_string(),
                    detail: Some(format!("startup script .{ext}")),
                });
                if hits.len() >= 8 {
                    break;
                }
            }
        }
    }

    hits
}

#[cfg(windows)]
fn windows_startup_dir() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA").map(PathBuf::from)?;
    Some(
        appdata
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup"),
    )
}

#[cfg(windows)]
fn scan_windows_mining_processes() -> Vec<SuspiciousActivityHit> {
    use crate::process::hidden_command;
    use std::process::Stdio;

    let output = hidden_command("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_Process | Select-Object -First 80 Name,ExecutablePath,CommandLine | ConvertTo-Csv -NoTypeInformation",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else {
        return vec![];
    };
    if !output.status.success() {
        return vec![];
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_process_csv_hits(&text)
}

fn parse_process_csv_hits(text: &str) -> Vec<SuspiciousActivityHit> {
    let mut hits = Vec::new();
    let mut lines = text.lines();
    let _header = lines.next();
    for line in lines {
        let cols = parse_csv_line(line);
        if cols.len() < 1 {
            continue;
        }
        let name = cols.first().map(|s| s.as_str()).unwrap_or("").trim();
        if name.is_empty() || name.eq_ignore_ascii_case("name") {
            continue;
        }
        let path = cols.get(1).map(|s| s.as_str()).filter(|s| !s.is_empty());
        let cmdline = cols.get(2).map(|s| s.as_str()).filter(|s| !s.is_empty());
        if looks_mining_like(name, path, cmdline) {
            hits.push(SuspiciousActivityHit {
                name: name.to_string(),
                location: path.unwrap_or("(path unknown)").to_string(),
                detail: cmdline.map(truncate_cmd),
            });
        }
        if hits.len() >= 8 {
            break;
        }
    }
    hits
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                out.push(cur.clone());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

#[cfg(target_os = "linux")]
fn scan_linux_persistence() -> Vec<SuspiciousActivityHit> {
    let mut hits = Vec::new();
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return hits;
    };
    let dir = home.join(".config/autostart");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return hits;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let exec = text.lines().find_map(|l| l.trim().strip_prefix("Exec="));
        if let Some(exec) = exec {
            if looks_script_host_command(exec)
                || SCRIPT_EXTS.iter().any(|ext| {
                    exec.to_ascii_lowercase().contains(&format!(".{ext}"))
                })
            {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("entry")
                    .to_string();
                hits.push(SuspiciousActivityHit {
                    name,
                    location: path.display().to_string(),
                    detail: Some(truncate_cmd(exec)),
                });
            }
        }
        if hits.len() >= 8 {
            break;
        }
    }
    hits
}

#[cfg(target_os = "macos")]
fn scan_macos_persistence() -> Vec<SuspiciousActivityHit> {
    let mut hits = Vec::new();
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return hits;
    };
    let dir = home.join("Library/LaunchAgents");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return hits;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("plist") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        if looks_script_host_command(&text)
            || text.to_ascii_lowercase().contains("osascript")
            || SCRIPT_EXTS.iter().any(|ext| text.to_ascii_lowercase().contains(&format!(".{ext}")))
        {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("agent")
                .to_string();
            hits.push(SuspiciousActivityHit {
                name,
                location: path.display().to_string(),
                detail: Some("LaunchAgent references script host / script file".to_string()),
            });
        }
        if hits.len() >= 8 {
            break;
        }
    }
    hits
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn scan_unix_mining_processes() -> Vec<SuspiciousActivityHit> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-eo", "comm=,args="])
        .output();
    let Ok(output) = output else {
        return vec![];
    };
    if !output.status.success() {
        return vec![];
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut hits = Vec::new();
    for line in text.lines().take(120) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (name, cmdline) = match line.split_once(char::is_whitespace) {
            Some((n, rest)) => (n, rest.trim()),
            None => (line, ""),
        };
        if looks_mining_like(name, None, Some(cmdline)) {
            hits.push(SuspiciousActivityHit {
                name: name.to_string(),
                location: "(process sample)".to_string(),
                detail: Some(truncate_cmd(cmdline)),
            });
        }
        if hits.len() >= 8 {
            break;
        }
    }
    hits
}

fn truncate_cmd(cmd: &str) -> String {
    let trimmed = cmd.trim();
    if trimmed.len() <= 120 {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..117])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_script_host_commands() {
        assert!(looks_script_host_command(
            r#"powershell -NoP -Enc WwBTAHkAcwB0AGUAbQAuAE4AZQB0AC4AUwBlAHIAdgBpAGMAZQBQAG8AaQBuAHQA"#,
        ));
        assert!(looks_script_host_command(
            r#"C:\Windows\System32\wscript.exe C:\Users\a\AppData\Local\Temp\x.vbs"#,
        ));
        assert!(looks_script_host_command("cmd /c powershell -File x.ps1"));
        assert!(!looks_script_host_command(r#"C:\Program Files\App\app.exe"#));
    }

    #[test]
    fn detects_mining_like() {
        assert!(looks_mining_like("xmrig", Some(r"C:\Tools\xmrig.exe"), None));
        assert!(looks_mining_like(
            "worker",
            Some(r"C:\Users\a\AppData\Local\Temp\worker.exe"),
            Some("--algo=rx -o stratum+tcp://pool.example:3333"),
        ));
        assert!(!looks_mining_like(
            "chrome",
            Some(r"C:\Program Files\Google\Chrome\chrome.exe"),
            Some("chrome.exe"),
        ));
    }

    #[test]
    fn settings_require_consent() {
        let dir = std::env::temp_dir().join(format!(
            "knottrace-suspicious-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(save_suspicious_activity_settings(
            &dir,
            &SuspiciousActivitySettings {
                enabled: true,
                consent_ack: false,
            },
        )
        .is_err());
        let ok = save_suspicious_activity_settings(
            &dir,
            &SuspiciousActivitySettings {
                enabled: true,
                consent_ack: true,
            },
        )
        .unwrap();
        assert!(ok.enabled && ok.consent_ack);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_csv_handles_quotes() {
        let line = r#""xmrig.exe","C:\Temp\xmrig.exe","xmrig --algo=rx -o stratum+tcp://x""#;
        let cols = parse_csv_line(line);
        assert_eq!(cols[0], "xmrig.exe");
        assert!(cols[2].contains("stratum"));
    }
}
