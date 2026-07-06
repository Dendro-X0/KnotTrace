# Architecture

## Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri v2 |
| UI | React 19, Tailwind CSS v4, shadcn/ui |
| Core engine | Rust (`network-core`) |
| History | SQLite (`rusqlite`, bundled) |
| HTTP probes | `reqwest` (rustls) |

## Crate boundaries

### `network-core`

Single source of truth for network logic. No UI dependencies.

| Module | Responsibility |
|--------|----------------|
| `env` | Interface, gateway, proxy, DNS server detection |
| `probe` | Latency and DNS measurements |
| `score` | Health grade from probe results |
| `diagnosis` | Bottleneck hints from a `HealthReport` |
| `stability` | Bufferbloat-lite and MTU hints |
| `tor` | Tor SOCKS detection |
| `dns_integrity` | Local vs trusted DNS comparison |
| `assist` | DNS Assist recommend/apply/restore |
| `connect` | Clash API, proxy groups, path comparison |
| `protect` | Trust, alerts, auto-protect evaluation |
| `benchmark` | Snapshot persistence |
| `throughput` | On-demand throughput samples |
| `store` | SQLite history |

`run_health_check()` orchestrates environment detection, probes, scoring, DNS integrity, stability, and diagnosis. Throughput and proxy delay comparison are **not** in the background path.

### `apps/desktop`

Thin Tauri command layer + React dashboard.

| Tauri module | Commands |
|--------------|----------|
| `monitor` | Health checks, background poll, tray |
| `assist` | DNS Assist |
| `connect` | Connect Assist + proxy comparison |
| `protect` | Protect settings and status |
| `benchmark` | Snapshots and diagnosis read |
| `throughput` | Throughput settings and on-demand test |
| `dns_integrity` | Integrity settings |

Frontend state centralizes in `useCompanion.ts`; pages live under `components/pages/`.

## Data flow

```
Background monitor / manual check
        │
        ▼
  run_health_check()  ──► HealthReport ──► SQLite history
        │                      │
        │                      ├──► diagnosis hints
        │                      ├──► protect evaluation
        │                      └──► UI (events + invoke)
        │
User-triggered (not in monitor)
        ├── throughput probe
        ├── proxy path comparison
        ├── DNS / Connect apply
        └── benchmark snapshot save
```

## Specs

Design notes and roadmap: [`specs/backend/`](../specs/backend/).
