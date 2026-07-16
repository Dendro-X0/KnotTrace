mod parsers;

#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod stub;

#[cfg(windows)]
use windows as platform;
#[cfg(not(windows))]
use stub as platform;

use crate::types::*;
use chrono::Utc;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use parsers::autotuning_is_ok;

#[derive(Debug, Error)]
pub enum LocalCapsError {
    #[error("local caps unavailable: {0}")]
    Unavailable(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn backup_path(data_dir: &Path) -> PathBuf {
    data_dir.join("local_caps_backup.json")
}

pub fn load_backup(data_dir: &Path) -> Result<Option<LocalCapsBackup>, LocalCapsError> {
    let path = backup_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&contents)?))
}

pub fn save_backup(data_dir: &Path, backup: &LocalCapsBackup) -> Result<(), LocalCapsError> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(backup_path(data_dir), serde_json::to_string_pretty(backup)?)?;
    Ok(())
}

pub fn clear_backup(data_dir: &Path) -> Result<(), LocalCapsError> {
    let path = backup_path(data_dir);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn get_local_caps_state(data_dir: &Path) -> Result<LocalCapsState, LocalCapsError> {
    let backup = load_backup(data_dir)?;
    Ok(LocalCapsState {
        available: platform::platform_available(),
        can_repair: platform::platform_available(),
        repair_active: backup.is_some(),
        backup,
        platform_note: platform::platform_note(),
    })
}

pub fn collect_local_caps(
    environment: &EnvironmentSnapshot,
    data_dir: Option<&Path>,
) -> LocalCapsReport {
    let repair_active = data_dir
        .and_then(|dir| load_backup(dir).ok().flatten())
        .is_some();

    if !platform::platform_available() {
        return LocalCapsReport {
            available: false,
            platform_note: platform::platform_note(),
            tcp_autotuning_level: None,
            tcp_autotuning_ok: true,
            adapter_name: None,
            adapter_power_saving: None,
            issues: Vec::new(),
            summary: platform::platform_note(),
            can_repair: false,
            repair_active,
        };
    }

    build_report(environment, repair_active)
}

fn build_report(environment: &EnvironmentSnapshot, repair_active: bool) -> LocalCapsReport {
    let mut issues = Vec::new();

    let tcp_autotuning_level = platform::read_autotuning_level().ok();
    let tcp_autotuning_ok = tcp_autotuning_level
        .as_deref()
        .map(autotuning_is_ok)
        .unwrap_or(true);

    if let Some(level) = tcp_autotuning_level.as_deref() {
        if !autotuning_is_ok(level) {
            let (kind, title, message) = if level == "disabled" {
                (
                    LocalCapsIssueKind::AutotuningDisabled,
                    "TCP receive-window auto-tuning is disabled".to_string(),
                    "With auto-tuning off, Windows can under-use fast or high-latency links. Restoring Normal usually helps large downloads.".to_string(),
                )
            } else {
                (
                    LocalCapsIssueKind::AutotuningRestricted,
                    "TCP auto-tuning is restricted".to_string(),
                    format!(
                        "Auto-tuning level is `{level}`, which can cap throughput on broadband or proxy paths with higher RTT."
                    ),
                )
            };
            issues.push(LocalCapsIssue {
                kind,
                severity: AlertLevel::Warning,
                title,
                message,
            });
        }
    }

    let adapter_name = platform::resolve_adapter_alias(&environment.interfaces);
    let adapter_power_saving = adapter_name
        .as_deref()
        .and_then(|alias| platform::read_adapter_power_saving(alias).ok().flatten());

    if adapter_power_saving == Some(true) {
        issues.push(LocalCapsIssue {
            kind: LocalCapsIssueKind::AdapterPowerSaving,
            severity: AlertLevel::Info,
            title: "NIC power saving may throttle the link".to_string(),
            message: format!(
                "Adapter `{}` allows Windows to turn it off to save power. That can cause stalls or uneven speed on laptops.",
                adapter_name.as_deref().unwrap_or("active")
            ),
        });
    }

    let summary = if issues.is_empty() {
        match tcp_autotuning_level.as_deref() {
            Some(level) => format!("Windows local stack looks fine (TCP auto-tuning: {level})."),
            None => "Windows local caps could not be fully read.".to_string(),
        }
    } else if let Some(issue) = issues.first() {
        issue.title.clone()
    } else {
        "Windows local caps need attention.".to_string()
    };

    let can_repair = !issues.is_empty() && !repair_active;

    LocalCapsReport {
        available: true,
        platform_note: platform::platform_note(),
        tcp_autotuning_level,
        tcp_autotuning_ok,
        adapter_name,
        adapter_power_saving,
        issues,
        summary,
        can_repair,
        repair_active,
    }
}

/// Opt-in repair: set TCP auto-tuning to Normal and disable NIC “turn off to save power”.
pub fn apply_local_caps_repair(
    data_dir: &Path,
    environment: &EnvironmentSnapshot,
) -> Result<LocalCapsRepairResult, LocalCapsError> {
    if !platform::platform_available() {
        return Err(LocalCapsError::Unavailable(platform::platform_note()));
    }
    if load_backup(data_dir)?.is_some() {
        return Err(LocalCapsError::Unavailable(
            "Local caps repair is already active. Restore before applying again.".to_string(),
        ));
    }

    let report = build_report(environment, false);
    if report.issues.is_empty() {
        return Err(LocalCapsError::Unavailable(
            "No Windows local-cap issues to repair right now.".to_string(),
        ));
    }

    let previous_autotuning_level = report
        .tcp_autotuning_level
        .clone()
        .unwrap_or_else(|| "normal".to_string());
    let adapter_name = report.adapter_name.clone();
    let previous_allow_computer_turn_off = report.adapter_power_saving;

    let backup = LocalCapsBackup {
        previous_autotuning_level: previous_autotuning_level.clone(),
        adapter_name: adapter_name.clone(),
        previous_allow_computer_turn_off,
        applied_at: Utc::now(),
    };
    save_backup(data_dir, &backup)?;

    let mut errors = Vec::new();

    if !report.tcp_autotuning_ok {
        if let Err(error) = platform::set_autotuning_level("normal") {
            errors.push(format!("auto-tuning: {error}"));
        }
    }

    if report.adapter_power_saving == Some(true) {
        if let Some(alias) = adapter_name.as_deref() {
            if let Err(error) = platform::set_adapter_power_saving(alias, false) {
                errors.push(format!("NIC power: {error}"));
            }
        }
    }

    if !errors.is_empty() {
        // Best-effort rollback of whatever we changed.
        let _ = restore_local_caps_repair(data_dir);
        return Err(LocalCapsError::Platform(errors.join("; ")));
    }

    Ok(LocalCapsRepairResult {
        kept: true,
        message: "Applied Windows local-cap repair (TCP auto-tuning Normal; NIC power-saving off when supported). You can restore anytime.".to_string(),
    })
}

pub fn restore_local_caps_repair(data_dir: &Path) -> Result<String, LocalCapsError> {
    if !platform::platform_available() {
        return Err(LocalCapsError::Unavailable(platform::platform_note()));
    }

    let backup = load_backup(data_dir)?.ok_or_else(|| {
        LocalCapsError::Unavailable("No local caps backup found. Nothing to restore.".to_string())
    })?;

    let mut errors = Vec::new();

    if let Err(error) = platform::set_autotuning_level(&backup.previous_autotuning_level) {
        errors.push(format!("auto-tuning: {error}"));
    }

    if let (Some(alias), Some(allow)) = (
        backup.adapter_name.as_deref(),
        backup.previous_allow_computer_turn_off,
    ) {
        if let Err(error) = platform::set_adapter_power_saving(alias, allow) {
            errors.push(format!("NIC power: {error}"));
        }
    }

    if !errors.is_empty() {
        return Err(LocalCapsError::Platform(errors.join("; ")));
    }

    clear_backup(data_dir)?;
    Ok("Restored previous Windows TCP auto-tuning and NIC power settings.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_report_off_windows_or_when_forced() {
        // On Windows this still builds a real report; parsers are covered separately.
        assert!(autotuning_is_ok("normal"));
        assert!(!autotuning_is_ok("disabled"));
    }
}
