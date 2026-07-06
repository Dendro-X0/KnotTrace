# Network Diagnosis & Benchmark Roadmap (v0.8)

## Goal

Help users understand **where** their connection is limited (home, corporate, proxy/VPN) and whether assists actually helped — without promising ISP cap overrides or background speed tests.

Preserves v0.7 principles:

- observe-first
- reversible assists
- manual confirmation for changes
- low background overhead

## Problem framing

Users on a 500 Mbps plan may only see ~100 Mbps. Causes differ by environment:

| Environment | Common bottlenecks |
|-------------|------------------|
| Home | Wi‑Fi, router bufferbloat, 100M link, modem |
| Corporate | Full-tunnel VPN, MTU/MSS, QoS caps, mandatory proxy |
| Proxy/VPN | Slow node, rule misrouting, tunnel overhead |

This app should **diagnose likely bottlenecks** from existing probes and guide next steps — not clone Speedtest.net.

## Non-goals (v0.8)

- Guaranteed Mbps improvement claims
- Automatic throughput tests in background monitor
- Corporate policy bypass
- Built-in iperf3 servers or heavy download workloads
- MTU/MSS auto-repair

## Phased rollout

### Phase A — Diagnose (v0.8.0) ← **current**

1. **Bottleneck hints** from health report (gateway, internet, DNS, proxy, VPN, Wi‑Fi/cellular, integrity)
2. **Benchmark snapshots** — manual save of app metrics + optional Speedtest numbers + environment fingerprint for before/after assist comparison

**Exit criteria:** diagnosis on every health check; user can save/compare snapshots.

### Phase B — Stability probes (v0.8.1) ← **current**

3. **Bufferbloat-lite** — idle ping vs ping during small controlled download
4. **MTU hint** — DF ping / path size probe with user-facing guidance (no auto-fix)
5. **Tor environment** — detect SOCKS `9050`/`9150`, diagnosis hints for Tor latency/MTU

**Exit criteria:** stability probes on health check; Tor shown on Network page; diagnosis includes bufferbloat/MTU/Tor hints.

### Phase C — Throughput (v0.9)

5. On-demand download/upload sample (user-triggered, configurable size cap)
6. Proxy path comparison using Connect Assist node list

**Exit criteria:** throughput test runs on demand from Network page; proxy group delay comparison via Clash API; not wired into background monitor.

## Data model (Phase A)

### `NetworkDiagnosis`

- `summary: String`
- `primary_bottleneck: Option<BottleneckCategory>`
- `hints: Vec<BottleneckHint>`

### `BottleneckHint`

- `category`, `severity`, `title`, `message`, `suggestions: Vec<String>`

### `BenchmarkSnapshot`

- `id`, `timestamp`, `label`
- `environment_fingerprint`
- `health_score`, `probe_summary`, `dns_integrity_state`
- `external_speedtest: Option<{ download_mbps, upload_mbps, ping_ms, source }>`
- `notes: Option<String>`

Stored in `%LOCALAPPDATA%/NetworkCompanion/benchmark_snapshots.json` (max 20).

## UI (Phase A)

- **Overview:** Diagnosis card with primary bottleneck + hint list
- **Overview:** Benchmark panel — save snapshot, optional Speedtest fields, compare last two

## API (Phase A)

**network-core:**

- `diagnose_network(report: &HealthReport) -> NetworkDiagnosis`
- `save_benchmark_snapshot`, `list_benchmark_snapshots`, `delete_benchmark_snapshot`

**Tauri:**

- `get_network_diagnosis` (from last report or fresh check)
- `list_benchmark_snapshots`, `save_benchmark_snapshot`, `delete_benchmark_snapshot`

## Verification

### L1

- `cargo test -p network-core`
- `cargo check --workspace`
- `npm run build`

### L3 (manual)

- Wi‑Fi + proxy on → Wi‑Fi and proxy hints appear
- Save baseline snapshot → apply DNS → save after → compare metrics

## Atomic steps (execution order)

| Step | Deliverable | Status |
|------|-------------|--------|
| 1 | This roadmap spec | done |
| 2 | `diagnosis.rs` + types + unit tests | done |
| 3 | Wire diagnosis into `HealthReport` | done |
| 4 | `benchmark.rs` persistence + Tauri commands | done |
| 5 | Overview UI (diagnosis + benchmarks) | done |
| 6 | README v0.8 section | done |
| 7 | `stability.rs` bufferbloat + MTU probes | done |
| 8 | Tor detection + diagnosis hints | done |
| 9 | Network page stability + Tor UI | done |
| 10 | `throughput.rs` on-demand probe + settings | done |
| 11 | Proxy path comparison via Clash delay API | done |
| 12 | Network page throughput + proxy comparison UI | done |
