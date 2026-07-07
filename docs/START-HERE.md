# Start here

KnotTrace is a **Tauri v2 desktop app** for connection health monitoring, symptom-aware diagnosis, and small reversible network assists.

## What it does

| Page | Purpose |
|------|---------|
| **Overview** | Health score, fast/full check badge, diagnosis shapes, next steps, trends, benchmark snapshots |
| **DNS Assist** | Recommend and apply a faster DNS resolver (Windows apply only) |
| **Connect** | Discover Mihomo/sing-box API and switch proxy nodes |
| **Protect** | Trust level, shape-aware alerts, notifications, opt-in auto-apply |
| **Network** | Environment facts, probes, stability, throughput, proxy path comparison |

**Default posture:** observe first. The background monitor runs full health checks; manual checks use a faster profile. DNS, Connect, and routing changes stay manual unless you enable auto-protect. Proxy path pinpointing is detect-only — it does not switch nodes for you.

## First five minutes

1. Run `npm run tauri dev` (see [development.md](./development.md)) or install a release build (see [release.md](./release.md)).
2. Open **Overview** — wait for the first health check (tray shows status).
3. Read **diagnosis** hints — note the slowdown shape and confidence badges.
4. On **Network**, review environment, proxy path comparison, stability probes, and optional throughput sample.
5. Use **DNS** / **Connect** only when you want to apply a reversible change.

## Local data

Windows: `%LOCALAPPDATA%/KnotTrace/` (legacy `NetworkCompanion/` is still read if present)

| File | Contents |
|------|----------|
| `history.db` | Health check history (includes slowdown shape when available) |
| `dns_backup.json` | DNS Assist backup |
| `connect_config.json` | Proxy API settings |
| `protect_settings.json` | Protect policies |
| `dns_integrity_settings.json` | Integrity verification domains |
| `benchmark_snapshots.json` | Benchmark snapshots (max 20) |
| `throughput_settings.json` | Throughput sample sizes |

## Docs index

- [development.md](./development.md) — dev setup and verification commands
- [release.md](./release.md) — installer build and pre-push checklist
- [architecture.md](./architecture.md) — repo layout and module boundaries
- [slowdown-factors.md](./slowdown-factors.md) — cross-platform slowdown research
- [slow-speed-triage.md](./slow-speed-triage.md) — symptom-to-signal triage matrix
- [platform-support.md](./platform-support.md) — what works on each OS
- [../specs/backend/](../specs/backend/) — design specs and roadmap
- [github-ci.md](./github-ci.md) — GitHub Actions CI and release publishing

## Principles

- Observe-first; assists are reversible and manual by default
- Diagnosis ranks symptom shape and confidence — it does not auto-fix upstream proxy quality
- No background throughput tests; on-demand samples only
- DNS integrity detects mismatches; it does not auto-change DNS
- Does not bypass corporate policy or claim guaranteed speed improvements
