# Link facts (L2 design)

Status: **implemented** (band L2). See also [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

Surface **negotiated link facts** for the active path and related adapters so KnotTrace can diagnose local bottlenecks (capped Ethernet, Wi‑Fi preferred over available Ethernet, half-duplex) instead of only saying “internet is slow.”

## Data

`LinkFactsReport` on `HealthReport`:

- Per-adapter: name, kind, up, default route, `speed_mbps`, duplex, media string
- Derived `issues` with severity
- Human `summary`

## Platform readers

| OS | Source |
|----|--------|
| Windows | `Get-NetAdapter` (LinkSpeed, FullDuplex, MediaType) |
| Linux | `/sys/class/net/*/speed` + `duplex`; optional |
| macOS | `networksetup -getMedia` after device→service map |

Observe-only — no MTU/link mutation (L6 / later).

## Diagnosis rules

1. Ethernet up + negotiated ≤ 100 Mbps → warning (capped link / cable/port)
2. Ethernet half-duplex → warning
3. Default route is Wi‑Fi while another Ethernet adapter is up → prefer-Ethernet recommendation
4. Active Wi‑Fi alone → existing info-level Wi‑Fi path hint (enriched with bitrate when known)

## UI

- Network page: **Link** panel (active speed, duplex, issues)
- Overview: existing diagnosis panel picks up new hints / `LinkLocalIssue` when issues fire
