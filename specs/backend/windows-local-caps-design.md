# Windows local caps (L3 design)

Status: **implemented** (band L3). See also [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

Detect Windows settings that **cap TCP throughput on high-BDP links** and offer **opt-in, reversible repair**:

1. TCP receive-window **auto-tuning** disabled or highly restricted
2. Active NIC **power management** allowing the OS to turn the device off

## Non-goals

- RSS / offload / driver updates (detect-only later)
- Silent or Protect auto-repair (manual confirm only)
- macOS / Linux equivalents in this band

## Data

`LocalCapsReport` on `HealthReport` (all platforms; `available=false` off Windows).

Backup file: `%LOCALAPPDATA%/KnotTrace/local_caps_backup.json`

## Repair policy

| Action | Command surface | Rollback |
|--------|-----------------|----------|
| Set Internet TCP auto-tuning to `Normal` | `Set-NetTCPSetting` / `netsh` | restore previous level |
| Disable “Allow computer to turn off this device” on active adapter | `Set-NetAdapterPowerManagement` | restore previous |

Always backup first. Refuse second apply until restore. Never auto-run from Protect.

## Diagnosis

Issues feed `BottleneckCategory::LinkLocal` and can select `SlowdownShape::LinkLocalIssue` when severity is warning+.
