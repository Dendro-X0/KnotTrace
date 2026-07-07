# KnotTrace

**v1.1.1** — A minimalist desktop companion that monitors your connection, improves it automatically when safe, and stays out of your way.

KnotTrace watches network health in the background, applies reversible DNS and proxy improvements on untrusted or poor connections, and explains issues in plain language.

## What runs automatically

| Feature | Default behavior |
|---------|------------------|
| **Health monitor** | On — checks every 15s when the network changes |
| **Smart protect** | On — alerts and optional auto-fixes |
| **Auto DNS improve** | On for untrusted/poor networks (Windows apply) |
| **Auto proxy switch** | On when Mihomo/sing-box API is available |
| **Diagnosis & alerts** | Automatic, plain-language summaries |
| **DNS integrity watch** | Automatic detection; no silent DNS overrides |
| **Throughput test** | Manual only (Network page) |
| **Benchmark snapshots** | Manual only (Overview) |

You can turn off any automatic behavior on the **Protect** page.

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
| [docs/github-ci.md](docs/github-ci.md) | GitHub Actions CI and release publishing |
| [docs/updater-signing.md](docs/updater-signing.md) | Signed updates, OSS keys, Android/iOS — **start here for release secrets** |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [specs/backend/](specs/backend/) | Design specs and roadmap (incl. [v1.2 connectivity recovery](specs/backend/connectivity-recovery-v1.2.md)) |

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
