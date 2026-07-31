# Wi‑Fi signal & radio facts (design — band B)

Status: **implemented** (band L7). Extends [link-facts-design.md](./link-facts-design.md).  
Related: [offline-local-repair-design.md](./offline-local-repair-design.md), [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

When the active (or any up) adapter is Wi‑Fi, surface **signal quality and radio context** so KnotTrace can say “weak Wi‑Fi” instead of only “internet is slow.”

## Data (extend `LinkAdapterFact`)

Optional fields (serde default / skip if none):

| Field | Meaning |
|-------|---------|
| `wifi_ssid` | Associated SSID when connected |
| `wifi_signal_percent` | 0–100 when platform reports percent |
| `wifi_signal_dbm` | RSSI dBm when available |
| `wifi_channel` | Channel number |
| `wifi_band` | `band_2_4` / `band_5` / `band_6` / `unknown` |
| `wifi_radio` | PHY string (e.g. 802.11ax) when known |

New `LinkIssueKind` values:

- `WifiWeakSignal` — percent ≤ 40 **or** dBm ≤ −70 when associated
- Keep existing `PreferEthernet` / `WifiActive`

## Platform readers

| OS | Source |
|----|--------|
| Windows | `netsh wlan show interfaces` (Signal, SSID, Channel, Radio type, Receive/Transmit rate) |
| Linux | `iw` / `/proc/net/wireless` / nl80211 when available |
| macOS | `system_profiler SPAirPortDataType` or `wdutil` / airport legacy — best-effort |

Observe-only in this band (no channel force, no driver install).

## Diagnosis / UI

- Link panel shows SSID, signal %, channel/band when Wi‑Fi active
- Diagnosis warning on weak signal with “move closer / use Ethernet” suggestions
- Recommendations category stays `LinkLocal`

## Non-goals

- Wi‑Fi analyzer / channel planner maps
- Capturing passwords or nearby AP lists beyond the associated BSS
- Guaranteeing better throughput by changing channels

## Proof

- Parser unit tests for `netsh wlan` sample output
- `cargo test -p network-core`
- Manual: weak-signal fixture or live Wi‑Fi shows badge + issue
