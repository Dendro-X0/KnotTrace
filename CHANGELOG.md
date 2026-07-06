# Changelog

All notable changes to Network Companion are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions align with `tauri.conf.json` and workspace `Cargo.toml`.

## [0.9.0] - 2026-07-06

### Added

- **On-demand throughput test** — user-triggered download/upload sample via Cloudflare endpoints (configurable size cap; not part of background monitor)
- **Proxy path comparison** — delay-test nodes in a Connect Assist proxy group via Mihomo/sing-box Clash API
- Network page **Throughput & proxy paths** panel

### Changed

- Tor and diagnosis copy generalized (no app-specific integrations)
- Release documentation under `docs/` and version alignment to 0.9.0

## [0.8.0] - 2026-07-06

### Added

- **Network diagnosis** — bottleneck hints from probes (Wi‑Fi, VPN, proxy, Tor, DNS, loss, integrity)
- **Benchmark snapshots** — save before/after metrics with optional external Speedtest Mbps fields
- **Stability probes** — bufferbloat-lite (latency under load) and path MTU hints on health checks
- **Tor awareness** — detect local Tor SOCKS (`127.0.0.1:9050` / `9150`) and surface routing context in diagnosis
- Overview diagnosis card and benchmark panel

## [0.7.0]

### Added

- **DNS integrity** — compare local resolver answers against trusted public DNS
- Integrity trend chart and configurable verification domains
- Protect alerts for suspicious DNS integrity states

## [0.6.0]

### Added

- **Auto-protect policies** — opt-in automatic DNS Assist and/or Connect Assist (10 min cooldown)
- Trend charts for health score, DNS latency, and internet latency from SQLite history

## [0.5.0]

### Added

- **Protect mode** — trust classification, protect alerts, desktop notifications, policy toggles

## [0.4.0]

### Added

- **Connect Assist** — Mihomo/sing-box discovery and recommended proxy switching

## [0.3.0]

### Added

- **DNS Assist** — reversible DNS resolver improvements on Windows
- Background health monitor, system tray, and desktop shell (Tauri v2 + React)

[0.9.0]: https://github.com/your-org/network-companion/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/your-org/network-companion/compare/v0.7.0...v0.8.0
