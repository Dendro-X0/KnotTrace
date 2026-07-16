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
| `stability` | Bufferbloat-lite and path MTU estimates |
| `link_facts` | Negotiated link speed/duplex and Wi‑Fi vs Ethernet issues |
| `local_caps` | Windows TCP auto-tuning / NIC power detect + opt-in repair |
| `mtu_assist` | Opt-in interface MTU clamp when fragmentation + tunnel evidenced |
| `tunnel_compare` | Direct / proxy / Tor path samples + expectation copy |
| `upstream_pool` | Claim grades for active-path vs recurring upstream-pool impairment |
| `egress` | Public IP probes and path consistency |
| `network_context` | Guest/public network and captive portal classification |
| `recommendations` | Plain-language guidance from current conditions |
| `tor` | Tor SOCKS detection |
| `dns_integrity` | Local vs trusted DNS comparison |
| `assist` | DNS Assist recommend/apply/restore |
| `dns_platform` | OS DNS apply/restore (Windows / macOS / Linux) |
| `connect` | Clash API, proxy groups, path comparison |
| `protect` | Trust, shape-aware alerts, auto-protect evaluation |
| `benchmark` | Snapshot persistence |
| `throughput` | On-demand throughput samples |
| `store` | SQLite history |

`run_health_check()` orchestrates environment detection, probes, scoring, DNS integrity, egress, network context, stability, reachability, proxy path, tunnel compare, link facts, local caps, snapshot upstream-pool claim, MTU assist eligibility, recommendations, and diagnosis. Check profile (`Fast` vs `Full`) controls probe depth. Throughput and Clash delay comparison are **not** in the background path. Desktop `monitor` re-evaluates upstream pool with history after each check.

### `apps/desktop`

Thin Tauri command layer + React dashboard.

| Tauri module | Commands |
|--------------|----------|
| `monitor` | Health checks (fast manual / full background), upstream-pool enrichment, background poll, tray |
| `assist` | DNS Assist |
| `local_caps` | Windows local-caps state / apply / restore |
| `mtu_assist` | MTU assist state / apply / restore |
| `connect` | Connect Assist + proxy comparison |
| `protect` | Protect settings and status |
| `benchmark` | Snapshots and diagnosis read |
| `throughput` | Throughput settings and on-demand test |
| `dns_integrity` | Integrity settings |

Frontend state centralizes in `useCompanion.ts`; pages live under `components/pages/`. Notable Network/Overview panels: `LinkFactsPanel`, `LocalCapsPanel`, `MtuAssistPanel`, `TunnelComparePanel`, `UpstreamPoolPanel`, `ProxyPathPanel`, `NetworkDiagnosisPanel`, `NextStepsPanel`, `TrendCharts`.

Scrolling uses a single `ScrollArea` component with the `.app-scroll` CSS class for consistent thin scrollbars.

## Data flow

```
Background monitor / manual check
        │
        ▼
  run_health_check(profile)  ──► HealthReport
        │                              │
        │                              ├──► link_facts / local_caps / mtu_assist
        │                              ├──► tunnel_compare / proxy_path_report
        │                              ├──► upstream_pool (snapshot; + history on desktop)
        │                              ├──► diagnosis + recommendations
        │                              ├──► protect evaluation
        │                              ├──► SQLite history (+ trend fields)
        │                              └──► UI (events + invoke)
        │
User-triggered (not in monitor)
        ├── throughput probe
        ├── proxy node delay comparison (Connect Assist)
        ├── DNS / Connect / local caps / MTU apply (manual confirm + backup)
        └── benchmark snapshot save
```

## Specs

Design notes and roadmap: [`specs/backend/`](../specs/backend/).

| Spec | Topic |
|------|-------|
| [local-first-improvement-roadmap.md](../specs/backend/local-first-improvement-roadmap.md) | L1–L6 product bands |
| [link-facts-design.md](../specs/backend/link-facts-design.md) | Link speed / duplex (L2) |
| [windows-local-caps-design.md](../specs/backend/windows-local-caps-design.md) | TCP / NIC power (L3) |
| [tunnel-path-compare-design.md](../specs/backend/tunnel-path-compare-design.md) | Tor/VPN expectations (L4) |
| [upstream-pool-quality-design.md](../specs/backend/upstream-pool-quality-design.md) | Pool claim grades (L5) |
| [mtu-assist-design.md](../specs/backend/mtu-assist-design.md) | Opt-in MTU clamp (L6) |
| [slow-speed-diagnosis-v1.3.x.md](../specs/backend/slow-speed-diagnosis-v1.3.x.md) | Symptom shapes, confidence, proxy path |
| [public-network-egress-v1.3.md](../specs/backend/public-network-egress-v1.3.md) | Public IP and guest network context |
| [connectivity-recovery-v1.2.md](../specs/backend/connectivity-recovery-v1.2.md) | Site reachability and smart protect recovery |
| [v1.5-silence-control-roadmap.md](../specs/backend/v1.5-silence-control-roadmap.md) | DND / quiet hours / audit (shipped) |
