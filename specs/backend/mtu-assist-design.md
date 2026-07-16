# Opt-in MTU assist (L6 design)

Status: **implemented** (band L6). See also [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

When **path fragmentation risk** is detected **and** a **tunnel/proxy path** is in use, offer an **opt-in, reversible interface MTU clamp** so large transfers stop black-holing. KnotTrace never silently changes MTU and never auto-applies from Protect.

## Eligibility (all required)

1. `stability.mtu.fragmentation_risk == true`
2. Tunnel evidenced: VPN tag, Tor detected, system proxy enabled, or tunnel-compare VPN/Tor flags
3. Current interface IPv4 MTU (when readable) is **greater than** the recommended clamp
4. Platform apply supported; no existing `mtu_assist_backup.json`

## Recommendation

`recommended_mtu = clamp(estimated_path_mtu, 1200..=1400)` when estimated path MTU is known; otherwise default **1400** when fragmentation risk is flagged under a tunnel.

## Non-goals

- Changing Tor/VPN vendor app internal MTU settings
- MSS clamping via firewall rules
- Silent or Protect auto-apply
- Claiming guaranteed speed increases

## Data

`MtuAssistReport` on `HealthReport`. Backup: `%LOCALAPPDATA%/KnotTrace/mtu_assist_backup.json` (or platform data dir).

| Field | Meaning |
|-------|---------|
| `fragmentation_risk` | From stability MTU probe |
| `tunnel_evidenced` | VPN / Tor / proxy context |
| `interface_name` | Active / default-route adapter alias |
| `current_mtu` | Read from OS when available |
| `recommended_mtu` | Clamp target |
| `can_repair` | Eligible + platform + no active backup |
| `repair_active` | Backup present |

## Repair policy

| Action | Windows | macOS | Linux |
|--------|---------|-------|-------|
| Read MTU | `Get-NetIPInterface` / `netsh` | `networksetup -getMTU` | sysfs / `ip` |
| Apply | `netsh … set subinterface mtu=N store=persistent` | `networksetup -setMTU` | `ip link set … mtu` |
| Restore | same surface with backed-up value | same | same |

Always backup first. Refuse second apply until restore. Admin/polkit may be required.

## Diagnosis / UI

- Diagnosis MTU hint points at Network → MTU assist when eligible
- Recommendations include opt-in copy; forbid implying auto-fix
- Network page panel with Apply / Restore

## Proof

- Unit tests: eligibility false without tunnel; true with fragmentation + VPN; recommended clamp bounds
- `cargo test -p network-core`
- Manual: lower MTU only after user confirm; restore returns previous value
