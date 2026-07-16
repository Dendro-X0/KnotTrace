# KnotTrace

**v1.6.0** — A desktop companion that monitors connection health, explains slowdowns in plain language, and offers **local-first, reversible** assists.

KnotTrace watches network health in the background, applies safe DNS/proxy assists when you opt in (or enable Protect auto-apply), and pinpoints whether issues are local (link, DNS, OS caps, MTU), tunnel overhead, or **upstream proxy-pool** quality — without fake “speed boosts.”

## What runs automatically

| Feature | Default behavior |
|---------|------------------|
| **Health monitor** | On — checks every 15s when the network changes (full profile) |
| **Manual health check** | Fast profile — quicker probes for on-demand checks |
| **Smart protect** | On — alerts and optional auto-fixes |
| **Do Not Disturb** | Off — when on, no system notifications (monitoring still runs); also in tray menu |
| **Notification digest** | Off — coalesce eligible OS alerts into one summary per cooldown |
| **Quiet hours** | Off — optional local time window that behaves like DND for notifications |
| **Auto DNS improve** | On for untrusted/poor networks (apply on Windows, macOS, Linux) |
| **Auto proxy switch** | Off by default — opt in on the Protect page |
| **Diagnosis & alerts** | Symptom-aware shapes, confidence, and plain-language hints |
| **Proxy path / upstream pool** | Detect-only — grades active path vs pool; never auto-rotates nodes |
| **Link / local caps / MTU** | Observe on every check; repairs are **manual confirm + backup** only |
| **DNS integrity watch** | Automatic detection; no silent DNS overrides |
| **Throughput test** | Manual only (Network page) |
| **Benchmark snapshots** | Manual only (Overview) |

You can turn off any automatic behavior on the **Protect** page.

## Highlights

### Local-first (L1–L6)

- **DNS Assist** — recommend/apply/restore on Windows, macOS, and Linux
- **Link facts** — negotiated speed/duplex; capped Ethernet / half-duplex / prefer-Ethernet hints
- **Windows local caps** — TCP auto-tuning and NIC power-saving; opt-in reversible repair
- **Tunnel compare** — Direct vs system proxy vs Tor with honest expectations (Tor is not for speed)
- **Upstream pool proof** — claim grades from differential reachability + history recurrence; no node thrashing advice when the pool looks poor
- **MTU assist** — opt-in interface MTU clamp when fragmentation risk and tunnel/proxy align

### Also in v1.5.x

- **Silence & control** — Do Not Disturb (app + tray), quiet hours, notification digest, auto-protect audit history
- **Global search** — `Ctrl+K` / `⌘K` to jump pages or run a check
- **Light/dark themes** — Light, Dark, or System mode with persisted preference

## Quick start

### Users

Download the installer from your release artifacts, or build locally — see [docs/release.md](docs/release.md).

### Developers

```bash
cd apps/desktop
npm install
npm run tauri dev
```

Full setup, verification commands, and repo layout: **[docs/START-HERE.md](docs/START-HERE.md)**

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/START-HERE.md](docs/START-HERE.md) | Onboarding, first five minutes, data paths |
| [docs/development.md](docs/development.md) | Dev setup and test commands |
| [docs/release.md](docs/release.md) | Installer build and pre-push checklist |
| [docs/architecture.md](docs/architecture.md) | Modules and data flow |
| [docs/platform-support.md](docs/platform-support.md) | OS feature matrix |
| [docs/slowdown-factors.md](docs/slowdown-factors.md) | Cross-platform causes of slow internet |
| [docs/slow-speed-triage.md](docs/slow-speed-triage.md) | Symptom-to-signal triage matrix |
| [local-first improvement](specs/backend/local-first-improvement-roadmap.md) | Local OS fixes first; honest Tor/VPN; proxy proof second |
| [link facts](specs/backend/link-facts-design.md) | Negotiated speed / Wi‑Fi vs Ethernet (L2) |
| [Windows local caps](specs/backend/windows-local-caps-design.md) | TCP auto-tuning / NIC power opt-in repair (L3) |
| [tunnel path compare](specs/backend/tunnel-path-compare-design.md) | Direct vs proxy vs Tor expectations (L4) |
| [upstream pool quality proof](specs/backend/upstream-pool-quality-design.md) | Prove proxy upstream/pool faults vs local (L5 A–C) |
| [MTU assist](specs/backend/mtu-assist-design.md) | Opt-in interface MTU clamp (L6) |
| [docs/github-ci.md](docs/github-ci.md) | GitHub Actions CI and release publishing |
| [docs/updater-signing.md](docs/updater-signing.md) | Signed updates — **start here for release secrets** |
| [docs/mobile-signing.md](docs/mobile-signing.md) | Android/iOS self-signed install policy and CI secrets |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [specs/backend/](specs/backend/) | Design specs and roadmap |
| [v1.5 silence & control](specs/backend/v1.5-silence-control-roadmap.md) | DND, tray silence, audit log (shipped in 1.5.0) |

## Verify before push

```bash
cargo test -p network-core
cargo check --workspace
cd apps/desktop && npm run build && npm run tauri build
```

On low-memory hosts, prefer `CARGO_BUILD_JOBS=1` for Rust commands.

## Local data

`%LOCALAPPDATA%/KnotTrace/` on Windows (migrates from `NetworkCompanion` if present). See [docs/START-HERE.md](docs/START-HERE.md) for backup file names (`dns_backup.json`, `local_caps_backup.json`, `mtu_assist_backup.json`, …).

## License

MIT
