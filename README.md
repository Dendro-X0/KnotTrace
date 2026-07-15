# KnotTrace

**v1.5.0** — A minimalist desktop companion that monitors your connection, improves it automatically when safe, and explains slowdowns in plain language.

KnotTrace watches network health in the background, applies reversible DNS and proxy improvements on untrusted or poor connections, and pinpoints whether issues are local, DNS-related, or on the proxy path.

## What runs automatically

| Feature | Default behavior |
|---------|------------------|
| **Health monitor** | On — checks every 15s when the network changes (full profile) |
| **Manual health check** | Fast profile — quicker probes for on-demand checks |
| **Smart protect** | On — alerts and optional auto-fixes |
| **Do Not Disturb** | Off — when on, no system notifications (monitoring still runs); also in tray menu |
| **Notification digest** | Off — coalesce eligible OS alerts into one summary per cooldown |
| **Quiet hours** | Off — optional local time window that behaves like DND for notifications |
| **Auto DNS improve** | On for untrusted/poor networks (Windows apply) |
| **Auto proxy switch** | Off by default — opt in on the Protect page |
| **Diagnosis & alerts** | Symptom-aware shapes, confidence, and plain-language hints |
| **Proxy path pinpointing** | Detect-only — compares proxy vs direct site reachability |
| **DNS integrity watch** | Automatic detection; no silent DNS overrides |
| **Throughput test** | Manual only (Network page) |
| **Benchmark snapshots** | Manual only (Overview) |

You can turn off any automatic behavior on the **Protect** page.

## Highlights in v1.5.x

- **Silence & control** — Do Not Disturb (app + tray), quiet hours, notification digest, and auto-protect audit history
- **Global search** — `Ctrl+K` / `⌘K` to jump pages or run a check
- **Light/dark themes** — explicit Light, Dark, or System mode with persisted preference
- **Silent background monitoring** — no console flash on Windows; notifications only when issues are detected
- **Slow-speed diagnosis** — classifies slowdown shape (DNS delay, proxy path, captive portal, bufferbloat, and more) with confidence
- **Proxy path report** — shows which sites fail only through your system proxy so you can switch nodes manually

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
| [docs/slowdown-factors.md](docs/slowdown-factors.md) | Cross-platform causes of slow internet |
| [docs/slow-speed-triage.md](docs/slow-speed-triage.md) | Symptom-to-signal triage matrix |
| [docs/platform-support.md](docs/platform-support.md) | OS feature matrix |
| [docs/github-ci.md](docs/github-ci.md) | GitHub Actions CI and release publishing |
| [docs/updater-signing.md](docs/updater-signing.md) | Signed updates — **start here for release secrets** |
| [docs/mobile-signing.md](docs/mobile-signing.md) | Android/iOS self-signed install policy and CI secrets |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [specs/backend/](specs/backend/) | Design specs and roadmap |
| [v1.5 silence & control](specs/backend/v1.5-silence-control-roadmap.md) | Next-version theme: DND, tray silence, audit log |

## Verify before push

```bash
cargo test -p network-core
cargo check --workspace
cd apps/desktop && npm run build && npm run tauri build
```

On low-memory hosts, prefer `CARGO_BUILD_JOBS=1` for Rust commands.

## Local data

`%LOCALAPPDATA%/KnotTrace/` on Windows (migrates from `NetworkCompanion` if present).

## License

MIT
