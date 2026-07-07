# Changelog

All notable changes to Network Companion are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions align with `tauri.conf.json` and workspace `Cargo.toml`.

## [1.3.0] - 2026-07-07

### Added

- **Public egress IP probes** — multi-provider IP detection with confidence scoring (Win/macOS/Linux)
- **Guest/public network context** — classifies hotel/café Wi-Fi, cellular, and captive portals
- **Network recommendations** — guidance panel without forced WiFi/VPN/proxy changes
- **Tor egress comparison** — compares system path vs Tor SOCKS when reachable
- Spec: [`specs/backend/public-network-egress-v1.3.md`](specs/backend/public-network-egress-v1.3.md)

### Changed

- **Protect defaults** — proxy node auto-switch and site-access recovery now **opt-in** (off by default)
- Guest Wi-Fi on private gateways is treated as **untrusted** even when tagged `home_lan`

## [1.2.0] - 2026-07-07

### Added

- **Site reachability probes** — HTTPS checks on verification domains to detect proxy/path blocks
- **Connectivity recovery (smart protect)** — auto DNS on integrity poisoning; auto proxy switch on site failures
- Protect toggles: `auto_recover_dns_integrity`, `auto_recover_site_access`
- Spec: [`specs/backend/connectivity-recovery-v1.2.md`](specs/backend/connectivity-recovery-v1.2.md)
- `scripts/setup-github-signing-secret.sh` / `.ps1` — configure `TAURI_SIGNING_PRIVATE_KEY` on GitHub

### Changed

- CI skips doc-only pushes; faster Linux Rust checks; Android release moved to manual workflow
- Signing docs: password secret optional when key has no passphrase

## [1.1.1] - 2026-07-07

### Added

- `scripts/fix-knottrace-icons.py` — regenerate Tauri icons without white canvas bleed

### Changed

- **Responsive layout** — sidebar, overview grids, and header adapt to narrow and wide window sizes
- **Flatter sidebar tabs** — single-row nav pills instead of tall two-line cards
- **Minimum window size** lowered to 380×520 for small-screen layouts

### Fixed

- **App icon** — title bar and taskbar no longer show white corners on the KnotTrace logo
- **Proxy-aware health checks** — internet probe degrades gracefully behind Clash/Mihomo system proxy

## [1.1.0] - 2026-07-06

### Added

- **Signed in-app updates** — download, verify (minisign), install, and restart via Tauri updater
- **OSS signing workflow** — `scripts/generate-updater-keys.sh`, `docs/updater-signing.md`, CI secrets
- **Android release job** (optional) — enable with repo variable `ENABLE_ANDROID_RELEASE=true`
- Mobile updater capability file for Android/iOS builds

### Changed

- Update banner offers **Install update** when signed `latest.json` is available; falls back to GitHub otherwise

## [1.0.0] - 2026-07-06

### Added

- **KnotTrace** rebrand — minimalist logo, new app name, window title, and tray
- **In-app update checker** — compares your version to the latest GitHub release
- **Smart protect defaults** — auto DNS and proxy improvements on untrusted/poor networks
- Proxy-aware internet probes (SOCKS5 fallback for Clash/Mihomo system proxy)

### Changed

- Version display in sidebar; installer filenames now match `tauri.conf.json` version
- Protect alerts use plain language; fewer manual action buttons when auto-fix is on
- Local data folder: `%LOCALAPPDATA%/KnotTrace/` (legacy `NetworkCompanion` still read)

## [0.9.1] - 2026-07-06

### Fixed

- Health checks no longer fail entirely when direct internet TCP is blocked by system proxy
- Internet latency measured via proxy path (SOCKS5) when Clash/Mihomo is active

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

[0.9.0]: https://github.com/Dendro-X0/network/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/Dendro-X0/network/compare/v0.7.0...v0.8.0
