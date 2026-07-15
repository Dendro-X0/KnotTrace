use crate::types::AutoProtectLogEntry;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_LOG_LINES: usize = 200;

#[derive(Debug, Error)]
pub enum AutoProtectLogError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn log_path(data_dir: &Path) -> PathBuf {
    data_dir.join("auto_protect_log.jsonl")
}

pub fn append_auto_protect_log(
    data_dir: &Path,
    entry: &AutoProtectLogEntry,
) -> Result<(), AutoProtectLogError> {
    std::fs::create_dir_all(data_dir)?;
    let path = log_path(data_dir);
    let line = serde_json::to_string(entry)?;

    let mut lines = if path.exists() {
        std::fs::read_to_string(&path)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    lines.push(line);
    if lines.len() > MAX_LOG_LINES {
        lines.drain(0..lines.len() - MAX_LOG_LINES);
    }

    let mut file = std::fs::File::create(&path)?;
    for stored in lines {
        writeln!(file, "{stored}")?;
    }

    Ok(())
}

pub fn list_auto_protect_log(
    data_dir: &Path,
    limit: usize,
) -> Result<Vec<AutoProtectLogEntry>, AutoProtectLogError> {
    let path = log_path(data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<AutoProtectLogEntry>(&line) {
            entries.push(entry);
        }
    }

    let start = entries.len().saturating_sub(limit);
    Ok(entries[start..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_entry(kind: &str) -> AutoProtectLogEntry {
        AutoProtectLogEntry {
            timestamp: Utc::now(),
            kind: kind.to_string(),
            success: true,
            message: "applied".to_string(),
            trigger: "untrusted".to_string(),
            check_reason: "scheduled".to_string(),
            rollback_hint: "Restore DNS Assist".to_string(),
        }
    }

    #[test]
    fn appends_and_lists_entries() {
        let dir = std::env::temp_dir().join(format!("knottrace-log-{}", uuid_like()));
        let _ = std::fs::remove_dir_all(&dir);
        append_auto_protect_log(&dir, &sample_entry("dns")).unwrap();
        append_auto_protect_log(&dir, &sample_entry("connect")).unwrap();

        let entries = list_auto_protect_log(&dir, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].kind, "dns");
        assert_eq!(entries[1].kind, "connect");

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn uuid_like() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
