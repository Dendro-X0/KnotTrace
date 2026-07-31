# Offline local repair companion (design — band A)

Status: **implemented** (band L8).  
Related: [wifi-signal-design.md](./wifi-signal-design.md), [mtu-assist-design.md](./mtu-assist-design.md), [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

When the machine has **local** connectivity problems (or the user wants a classic “repair” step), KnotTrace offers **opt-in, reversible-ish local actions** that work **without** a working internet path — closer to network repair tools, without silent stack nukes.

## Eligible actions (v1)

| Action | Purpose | Elevation | Platforms (v1) |
|--------|---------|-----------|----------------|
| **Flush DNS cache** | Clear poisoned / stale resolver cache | Usually none / low | Win / macOS / Linux |
| **Renew DHCP** | Rebind IPv4 address/gateway/DNS from LAN | May need admin | Win primary; best-effort elsewhere |
| **Restart adapter** | Disable+enable active NIC (stuck link) | Often admin | Win primary; best-effort elsewhere |

Each action returns a short result message and triggers a health re-check.

## Suggested when

`LocalRepairReport` lists actions with `recommended: bool` based on current report:

| Signal | Prefer |
|--------|--------|
| DNS integrity caution/suspicious or high DNS latency | Flush DNS |
| Gateway unreachable / high loss / no IPv4 on active iface | Renew DHCP |
| Adapter up but no gateway **or** link facts show Wi‑Fi weak + ethernet available | Restart adapter (and prefer Ethernet copy) |
| Internet unreachable + gateway OK | Do **not** claim local repair will fix ISP — show honest copy |

## Non-goals (explicit)

- Silent Winsock / TCP-IP stack reset
- Driver reinstall / Wi‑Fi channel force
- Protect auto-apply of any local repair
- Claiming repair restores Steam CDN or ISP outages

## Data

- `LocalRepairReport` on `HealthReport` (suggestions + last-run notes optional)
- Tauri commands: `run_local_repair_action { action }` → result + recheck
- No persistent “backup file” for flush DNS; DHCP/adapter restart are best-effort restore via renew/enable

## UI

Network page **Local repair** panel:

- Always visible (useful offline)
- Buttons disabled while running
- Each row: title, why recommended, Run
- Footer: “These fix local stack glitches — not your ISP or Steam CDN.”

## Offline posture

Health check already probes gateway locally. Local repair must not require proxy path / upstream pool / throughput. Internet probe failure is a **reason to show** local repair, not to hide it.

## Proof

- Unit tests: suggestion mapping from fixture reports
- Platform command wrappers return clear errors without panicking
- `cargo test -p network-core`; manual flush DNS on Windows
