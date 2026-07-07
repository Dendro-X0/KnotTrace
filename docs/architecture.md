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
| `diagnosis` | Symptom-aware slowdown shape, confidence, and bottleneck hints |
| `reachability` | HTTPS site probes, error taxonomy, proxy vs direct path report |
| `stability` | Bufferbloat-lite and MTU hints |
| `egress` | Public IP probes and path consistency |
| `network_context` | Guest/public network and captive portal classification |
| `recommendations` | Plain-language guidance from current conditions |
| `tor` | Tor SOCKS detection |
| `dns_integrity` | Local vs trusted DNS comparison |
| `assist` | DNS Assist recommend/apply/restore |
| `connect` | Clash API, proxy groups, path comparison |
| `protect` | Trust, shape-aware alerts, auto-protect evaluation |
| `benchmark` | Snapshot persistence |
| `throughput` | On-demand throughput samples |
| `store` | SQLite history |

`run_health_check()` orchestrates environment detection, probes, scoring, DNS integrity, egress IP checks, network-context classification, stability, reachability, recommendations, and diagnosis. Check profile (`Fast` vs `Full`) controls probe depth. Throughput and proxy delay comparison are **not** in the background path.

### `apps/desktop`

Thin Tauri command layer + React dashboard.

| Tauri module | Commands |
|--------------|----------|
| `monitor` | Health checks (fast manual / full background), background poll, tray |
| `assist` | DNS Assist |
| `connect` | Connect Assist + proxy comparison |
| `protect` | Protect settings and status |
| `benchmark` | Snapshots and diagnosis read |
| `throughput` | Throughput settings and on-demand test |
| `dns_integrity` | Integrity settings |

Frontend state centralizes in `useCompanion.ts`; pages live under `components/pages/`. Notable UI panels: `NetworkDiagnosisPanel`, `NextStepsPanel`, `ProxyPathPanel`, `TrendCharts`.

Scrolling uses a single `ScrollArea` component with the `.app-scroll` CSS class for consistent thin scrollbars.

## Data flow

```
Background monitor / manual check
        │
        ▼
  run_health_check(profile)  ──► HealthReport ──► SQLite history
        │                              │
        │                              ├──► diagnosis (shape + confidence)
        │                              ├──► proxy_path_report (when proxy on)
        │                              ├──► protect evaluation
        │                              └──► UI (events + invoke)
        │
User-triggered (not in monitor)
        ├── throughput probe
        ├── proxy node delay comparison (Connect Assist)
        ├── DNS / Connect apply
        └── benchmark snapshot save
```

## Specs

Design notes and roadmap: [`specs/backend/`](../specs/backend/).

| Spec | Topic |
|------|-------|
| [slow-speed-diagnosis-v1.3.x.md](../specs/backend/slow-speed-diagnosis-v1.3.x.md) | Symptom shapes, confidence, proxy path pinpointing |
| [public-network-egress-v1.3.md](../specs/backend/public-network-egress-v1.3.md) | Public IP and guest network context |
| [connectivity-recovery-v1.2.md](../specs/backend/connectivity-recovery-v1.2.md) | Site reachability and smart protect recovery |
