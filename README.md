# Network Companion

**v0.9.0** — A cross-platform desktop companion for connection health monitoring, diagnosis, and small reversible network improvements.

Understand how your path behaves across home networks, VPN/proxy tunnels, and Tor — then apply assists only when you choose to.

## Features

| Area | Highlights |
|------|------------|
| **Monitor** | Background health checks, system tray, desktop notifications |
| **Diagnosis** | Bottleneck hints (Wi‑Fi, VPN, proxy, Tor, DNS, loss, integrity) |
| **DNS** | Integrity checks, trends, reversible DNS Assist (**apply: Windows**) |
| **Connect** | Mihomo/sing-box discovery, node recommendation, delay comparison |
| **Protect** | Trust classification, alerts, opt-in auto-apply (off by default) |
| **Network** | Stability probes (bufferbloat, MTU), on-demand throughput sample |

**Posture:** observe first. DNS and proxy changes are manual unless you enable auto-protect.

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
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [specs/backend/](specs/backend/) | Design specs and roadmap |

## Verify before push

```bash
cargo test -p network-core
cargo check --workspace
cd apps/desktop && npm run build && npm run tauri build
```

On low-memory hosts, prefer `CARGO_BUILD_JOBS=1` for Rust commands.

## Local data

`%LOCALAPPDATA%/NetworkCompanion/` on Windows — history, assist backups, settings, benchmarks.

## License

MIT
