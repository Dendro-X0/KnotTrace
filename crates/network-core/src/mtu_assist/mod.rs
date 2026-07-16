mod parsers;

#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod stub;

#[cfg(windows)]
use windows as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
use stub as platform;

use crate::types::*;
use chrono::Utc;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use parsers::{recommend_mtu, tunnel_evidenced};

const DEFAULT_TUNNEL_MTU: u16 = 1400;

#[derive(Debug, Error)]
pub enum MtuAssistError {
    #[error("mtu assist unavailable: {0}")]
    Unavailable(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn backup_path(data_dir: &Path) -> PathBuf {
    data_dir.join("mtu_assist_backup.json")
}

pub fn load_backup(data_dir: &Path) -> Result<Option<MtuAssistBackup>, MtuAssistError> {
    let path = backup_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&contents)?))
}

pub fn save_backup(data_dir: &Path, backup: &MtuAssistBackup) -> Result<(), MtuAssistError> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(backup_path(data_dir), serde_json::to_string_pretty(backup)?)?;
    Ok(())
}

pub fn clear_backup(data_dir: &Path) -> Result<(), MtuAssistError> {
    let path = backup_path(data_dir);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn get_mtu_assist_state(data_dir: &Path) -> Result<MtuAssistState, MtuAssistError> {
    let backup = load_backup(data_dir)?;
    Ok(MtuAssistState {
        available: platform::platform_available(),
        can_repair: platform::platform_available(),
        repair_active: backup.is_some(),
        backup,
        platform_note: platform::platform_note(),
    })
}

pub fn collect_mtu_assist(
    report: &HealthReport,
    data_dir: Option<&Path>,
) -> MtuAssistReport {
    let repair_active = data_dir
        .and_then(|dir| load_backup(dir).ok().flatten())
        .is_some();

    let mtu_probe = report.stability.as_ref().and_then(|stability| stability.mtu.as_ref());
    let fragmentation_risk = mtu_probe.is_some_and(|mtu| mtu.fragmentation_risk);
    let estimated_path_mtu = mtu_probe.and_then(|mtu| mtu.estimated_path_mtu);
    let tunnel = tunnel_evidenced(report);
    let recommended_mtu = recommend_mtu(mtu_probe);

    let interface_name = platform::resolve_interface_name(&report.environment);
    let current_mtu = interface_name
        .as_deref()
        .and_then(|name| platform::read_mtu(name).ok());

    if !platform::platform_available() {
        return MtuAssistReport {
            available: false,
            platform_note: platform::platform_note(),
            fragmentation_risk,
            tunnel_evidenced: tunnel,
            interface_name,
            current_mtu,
            recommended_mtu,
            estimated_path_mtu,
            summary: platform::platform_note(),
            can_repair: false,
            repair_active,
        };
    }

    let eligible = fragmentation_risk && tunnel;
    let needs_clamp = match (current_mtu, recommended_mtu) {
        (Some(current), Some(recommended)) => current > recommended,
        (None, Some(_)) => eligible,
        _ => false,
    };

    let summary = if repair_active {
        format!(
            "MTU assist is active (interface clamped). Restore anytime to undo."
        )
    } else if eligible && needs_clamp {
        format!(
            "Path MTU looks low under a tunnel/proxy. Opt-in clamp of {} to {} bytes may reduce fragmentation stalls.",
            interface_name.as_deref().unwrap_or("the active interface"),
            recommended_mtu.unwrap_or(DEFAULT_TUNNEL_MTU)
        )
    } else if fragmentation_risk && !tunnel {
        "Low path MTU detected, but no tunnel/proxy context — KnotTrace will not offer an interface clamp without tunnel evidence.".to_string()
    } else if tunnel && !fragmentation_risk {
        "Tunnel/proxy is active and path MTU looks typical — no clamp recommended.".to_string()
    } else {
        "No MTU assist action indicated right now.".to_string()
    };

    let can_repair = eligible && needs_clamp && !repair_active && recommended_mtu.is_some();

    MtuAssistReport {
        available: true,
        platform_note: platform::platform_note(),
        fragmentation_risk,
        tunnel_evidenced: tunnel,
        interface_name,
        current_mtu,
        recommended_mtu,
        estimated_path_mtu,
        summary,
        can_repair,
        repair_active,
    }
}

pub fn apply_mtu_assist_repair(
    data_dir: &Path,
    report: &HealthReport,
) -> Result<MtuAssistRepairResult, MtuAssistError> {
    if !platform::platform_available() {
        return Err(MtuAssistError::Unavailable(platform::platform_note()));
    }
    if load_backup(data_dir)?.is_some() {
        return Err(MtuAssistError::Unavailable(
            "MTU assist is already active. Restore before applying again.".to_string(),
        ));
    }

    let snapshot = collect_mtu_assist(report, Some(data_dir));
    if !snapshot.can_repair {
        return Err(MtuAssistError::Unavailable(
            "MTU assist is not eligible right now (needs fragmentation risk + tunnel evidence).".to_string(),
        ));
    }

    let interface_name = snapshot.interface_name.clone().ok_or_else(|| {
        MtuAssistError::Unavailable("No active interface found for MTU assist.".to_string())
    })?;
    let recommended = snapshot.recommended_mtu.ok_or_else(|| {
        MtuAssistError::Unavailable("No recommended MTU available.".to_string())
    })?;
    let previous = snapshot
        .current_mtu
        .or_else(|| platform::read_mtu(&interface_name).ok())
        .ok_or_else(|| {
            MtuAssistError::Unavailable(
                "Could not read current interface MTU before applying.".to_string(),
            )
        })?;

    if previous <= recommended {
        return Err(MtuAssistError::Unavailable(
            "Current MTU is already at or below the recommended clamp.".to_string(),
        ));
    }

    let backup = MtuAssistBackup {
        interface_name: interface_name.clone(),
        previous_mtu: previous,
        applied_mtu: recommended,
        applied_at: Utc::now(),
    };
    save_backup(data_dir, &backup)?;

    if let Err(error) = platform::set_mtu(&interface_name, recommended) {
        let _ = clear_backup(data_dir);
        return Err(MtuAssistError::Platform(error));
    }

    Ok(MtuAssistRepairResult {
        kept: true,
        message: format!(
            "Set `{interface_name}` MTU from {previous} to {recommended}. Restore anytime to undo. Admin approval may have been required."
        ),
    })
}

pub fn restore_mtu_assist_repair(data_dir: &Path) -> Result<String, MtuAssistError> {
    if !platform::platform_available() {
        return Err(MtuAssistError::Unavailable(platform::platform_note()));
    }

    let backup = load_backup(data_dir)?.ok_or_else(|| {
        MtuAssistError::Unavailable("No MTU assist backup found. Nothing to restore.".to_string())
    })?;

    platform::set_mtu(&backup.interface_name, backup.previous_mtu)
        .map_err(MtuAssistError::Platform)?;
    clear_backup(data_dir)?;

    Ok(format!(
        "Restored `{}` MTU to {}.",
        backup.interface_name, backup.previous_mtu
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use super::parsers::{MAX_RECOMMENDED_MTU, MIN_RECOMMENDED_MTU};

    fn base_report(tunnel: bool, fragmentation: bool) -> HealthReport {
        let mut env = EnvironmentSnapshot {
            hostname: "host".to_string(),
            interfaces: vec![NetworkInterface {
                name: "eth0".to_string(),
                friendly_name: Some("Ethernet".to_string()),
                kind: LinkKind::Ethernet,
                is_up: true,
                ipv4: vec!["192.168.1.10".to_string()],
                gateway: Some("192.168.1.1".to_string()),
                is_default_route: true,
            }],
            dns_servers: Vec::new(),
            proxy: ProxySettings {
                enabled: tunnel,
                server: Some("127.0.0.1:7890".to_string()),
                source: "test".to_string(),
            },
            tags: if tunnel {
                vec![EnvironmentTag::Vpn]
            } else {
                Vec::new()
            },
            default_gateway: Some("192.168.1.1".to_string()),
            active_interface: Some("Ethernet".to_string()),
            tor: default_tor_status(),
        };
        if !tunnel {
            env.proxy.enabled = false;
            env.tags.clear();
        }

        HealthReport {
            timestamp: Utc::now(),
            environment: env,
            probe: ProbeResult {
                gateway: None,
                internet: Some(LatencySample {
                    target: "1.1.1.1".to_string(),
                    avg_ms: 20.0,
                    loss_pct: 0.0,
                }),
                dns: Vec::new(),
                duration_ms: 1,
            },
            score: HealthScore {
                score: 80,
                grade: HealthGrade::Good,
                summary: "ok".to_string(),
                reasons: Vec::new(),
            },
            dns_integrity: None,
            diagnosis: None,
            stability: Some(StabilityProbeResult {
                bufferbloat: None,
                mtu: Some(MtuProbe {
                    estimated_path_mtu: Some(1328),
                    recommended_tcp_mss: Some(1288),
                    fragmentation_risk: fragmentation,
                    summary: "low mtu".to_string(),
                }),
                duration_ms: 1,
            }),
            site_reachability: None,
            egress: None,
            network_context: None,
            recommendations: None,
            proxy_path_report: None,
            link_facts: None,
            local_caps: None,
            mtu_assist: None,
            tunnel_compare: None,
            upstream_pool: None,
        }
    }

    #[test]
    fn refuses_without_tunnel() {
        let report = base_report(false, true);
        assert!(!tunnel_evidenced(&report));
        let assist = collect_mtu_assist(&report, None);
        assert!(assist.fragmentation_risk);
        assert!(!assist.tunnel_evidenced);
        assert!(!assist.can_repair);
    }

    #[test]
    fn eligible_with_tunnel_and_fragmentation() {
        let report = base_report(true, true);
        assert!(tunnel_evidenced(&report));
        let recommended = recommend_mtu(report.stability.as_ref().and_then(|s| s.mtu.as_ref()));
        assert_eq!(recommended, Some(1328));
    }

    #[test]
    fn recommend_clamps_bounds() {
        let high = MtuProbe {
            estimated_path_mtu: Some(1500),
            recommended_tcp_mss: None,
            fragmentation_risk: true,
            summary: String::new(),
        };
        let low = MtuProbe {
            estimated_path_mtu: Some(1000),
            recommended_tcp_mss: None,
            fragmentation_risk: true,
            summary: String::new(),
        };
        assert_eq!(recommend_mtu(Some(&high)), Some(MAX_RECOMMENDED_MTU));
        assert_eq!(recommend_mtu(Some(&low)), Some(MIN_RECOMMENDED_MTU));
        assert_eq!(
            recommend_mtu(Some(&MtuProbe {
                estimated_path_mtu: None,
                recommended_tcp_mss: None,
                fragmentation_risk: true,
                summary: String::new(),
            })),
            Some(DEFAULT_TUNNEL_MTU)
        );
    }
}
