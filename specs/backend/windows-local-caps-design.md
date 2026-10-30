# Windows local caps (L3 design)

Status: **implemented** (Windows L3 repair + Band A2 Linux/macOS observe). See also [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

Detect OS settings that **cap TCP throughput on high-BDP links**. On Windows, also offer **opt-in, reversible repair**:

1. TCP receive-window **auto-tuning** disabled or highly restricted  
2. Active NIC **power management** allowing the OS to turn the device off  

## Non-goals

- RSS / offload / driver updates (detect-only later)
- Silent or Protect auto-repair (manual confirm only)
- macOS / Linux **repair** (observe only in Band A2)

## Data

`LocalCapsReport` on `HealthReport`:

| OS | `available` | `can_repair` | Signals |
|----|-------------|--------------|---------|
| Windows | true | true when issues | PowerShell / netsh auto-tuning + NIC power |
| Linux | true | false | `/proc` window scaling + `tcp_rmem`; sysfs `power/control` |
| macOS | true | false | `sysctl` `doautorcvbuf` / `doautosndbuf` |

Backup file (Windows repair only): `%LOCALAPPDATA%/KnotTrace/local_caps_backup.json`

## Repair policy

| Action | Command surface | Rollback |
|--------|-----------------|----------|
| Set Internet TCP auto-tuning to `Normal` | `Set-NetTCPSetting` / `netsh` | restore previous level |
| Disable “Allow computer to turn off this device” on active adapter | `Set-NetAdapterPowerManagement` | restore previous |

Always backup first. Refuse second apply until restore. Never auto-run from Protect.

## Diagnosis

Issues feed `BottleneckCategory::LinkLocal` and can select `SlowdownShape::LinkLocalIssue` when severity is warning+.
