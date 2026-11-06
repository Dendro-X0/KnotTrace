# KnotTrace

**v1.10.1** — Desktop companion for connection health, Path Truth diagnosis, and **local-first, reversible** assists — without fake “speed boosts.”

**Trying it out?** Start here → **[docs/COMMUNITY.md](docs/COMMUNITY.md)** (install, first five minutes, diagnostic export, what it is / is not).

## At a glance

| | |
|--|--|
| **Job** | Show *what kind of problem*, *where the path breaks*, and *safe next steps* |
| **Posture** | Observe-first · indicators with confidence · opt-in assists |
| **Not** | Mbps guarantees · AV/EDR · “you are unmonitored” |
| **OS matrix** | [docs/platform-support.md](docs/platform-support.md) |
| **Releases** | [GitHub Releases](https://github.com/Dendro-X0/KnotTrace/releases) |
| **Feedback** | [New issue](https://github.com/Dendro-X0/KnotTrace/issues/new/choose) (bug or feedback templates) |

## What runs automatically

| Feature | Default behavior |
|---------|------------------|
| **Health monitor** | On — checks when the network changes (full profile) |
| **Manual health check** | Fast profile |
| **Protect alerts** | On — optional auto DNS/proxy only when you enable it |
| **Do Not Disturb / quiet hours / digest** | Off |
| **Auto DNS improve** | On for untrusted/poor networks when Protect allows |
| **Auto proxy switch** | Off |
| **Proxy path / upstream pool** | Detect-only — never auto-rotates nodes |
| **Link / local caps / MTU** | Observe every check; repairs need manual confirm + backup |
| **Throughput / benchmarks** | Manual only |

Turn off automation on the **Protect** page.

## Highlights

- **Path Truth** — path break, integrity board, diagnostic export, local/cloud AI explain, egress timeline, CLI `network-probe check --json`
- **Local-first assists** — DNS (Win/macOS/Linux), Connect (Clash API), MTU clamp, Windows local-caps repair; Linux/macOS local-caps **observe**
- **Integrity depth** — hosts/proxy change timeline, WAN/LAN listener scope, Wi‑Fi noise/retries where OS allows, opt-in process↔egress, startup-agent, and bounded **script-risk** scans (indicators only; not AV)
- **Quick / Advanced UI** — default Quick; Settings for theme, language (en / zh-Hans), density, and monitor
- **Path Truth Quick honesty** — ranked next steps; captive/bufferbloat plain copy; tunnel “slower is expected” when local looks OK
- **CompanionShell** — thin check + export surface: **desktop preview** (`?companion=1`) and mobile packaging (`cfg(mobile)`); **device proof pending** (not a claimed phone product yet)

## Quick start

### Users

1. Download a desktop installer from [Releases](https://github.com/Dendro-X0/KnotTrace/releases)  
2. Follow **[docs/COMMUNITY.md](docs/COMMUNITY.md)** for first five minutes and how to export a diagnostic JSON  

### Developers

```bash
cd apps/desktop
npm install
npm run tauri dev
```

Cheap pre-push gate (mirrors CI):

```bash
./scripts/reliability-gate.sh
```

Full setup: **[docs/START-HERE.md](docs/START-HERE.md)** · **[docs/development.md](docs/development.md)**

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/COMMUNITY.md](docs/COMMUNITY.md) | **Public tryout kit** |
| [docs/START-HERE.md](docs/START-HERE.md) | Maintainer onboarding + data paths |
| [docs/development.md](docs/development.md) | Dev setup + R1 reliability gate |
| [docs/release.md](docs/release.md) | Installer build |
| [docs/platform-support.md](docs/platform-support.md) | OS feature matrix |
| [docs/product-plan-share-patch-1-10-1-2026-11.md](docs/product-plan-share-patch-1-10-1-2026-11.md) | Share patch **v1.10.1** (ready to tag) |
| [docs/product-plan-desktop-trust-path-truth-2026-11.md](docs/product-plan-desktop-trust-path-truth-2026-11.md) | Desktop trust + Path Truth Quick (complete @ v1.10.1) |
| [docs/product-plan-community-readiness-2026-11.md](docs/product-plan-community-readiness-2026-11.md) | Community readiness (complete @ v1.9.1) |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [specs/backend/](specs/backend/) | Design specs |

## Local data

`%LOCALAPPDATA%/KnotTrace/` on Windows (migrates from `NetworkCompanion` if present). See [docs/START-HERE.md](docs/START-HERE.md).

## License

MIT
