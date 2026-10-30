# Start here

KnotTrace is a **Tauri v2 desktop app** for connection health monitoring, symptom-aware diagnosis, and **local-first** reversible network assists.

Product direction: fix what is on the machine (DNS, link, OS caps, MTU) first; tell the truth about Tor/VPN overhead; **prove** when a proxy upstream/pool is the limit — do not fake Mbps gains. See [local-first-improvement-roadmap.md](../specs/backend/local-first-improvement-roadmap.md).

## What it does

| Page | Purpose |
|------|---------|
| **Overview** | Health score, fast/full check, diagnosis, **Path break**, **Integrity**, **Egress timeline**, local/cloud **Explain**, diagnostic export, upstream pool, next steps, trends, benchmarks |
| **DNS Assist** | Recommend and apply a faster DNS resolver (Windows, macOS, Linux) |
| **Connect** | Discover Mihomo/sing-box API and switch proxy nodes (manual / Protect opt-in) |
| **Protect** | Trust level, alerts, notifications, opt-in auto-apply, **process↔egress** opt-in |
| **Network** | Environment, Link (+ Wi‑Fi), Local repair, Local caps (Windows), MTU assist, Path break, Integrity, Egress timeline, Tunnel compare, proxy path + upstream pool, probes, throughput |

**Default posture:** observe first. The background monitor runs full health checks; manual checks use a faster profile. DNS, Connect, MTU, and local-caps changes stay manual unless you enable Protect auto-apply (DNS/proxy only — **never** MTU or local caps). Proxy path / upstream pool proof is detect-only — it does not switch nodes for you.

## First five minutes

1. Run `npm run tauri dev` (see [development.md](./development.md)) or install a release build (see [release.md](./release.md)).
2. Open **Overview** — wait for the first health check (tray shows status).
3. Read **diagnosis** hints — note the slowdown shape, confidence, and any upstream pool claim.
4. On **Network**, review **Link**, **Local caps** (Windows), **MTU assist**, **Upstream pool proof**, **Tunnel compare**, proxy path comparison, stability probes, and optional throughput.
5. Use **DNS** / **Connect** only when you want a reversible change (DNS apply works on Windows, macOS, and Linux).

## Local data

Windows: `%LOCALAPPDATA%/KnotTrace/` (legacy `NetworkCompanion/` is still read if present)

| File | Contents |
|------|----------|
| `history.db` | Health check history (includes slowdown shape and upstream claim fields when available) |
| `dns_backup.json` | DNS Assist backup |
| `local_caps_backup.json` | Windows local-caps repair backup |
| `mtu_assist_backup.json` | MTU assist previous interface MTU |
| `connect_config.json` | Proxy API settings |
| `protect_settings.json` | Protect policies |
| `dns_integrity_settings.json` | Integrity verification domains |
| `process_egress_settings.json` | Opt-in process↔egress correlation |
| `ai_explain_settings.json` / `ai_explain_api_key.txt` | Cloud AI explain (opt-in; key file separate) |
| `benchmark_snapshots.json` | Benchmark snapshots (max 20) |
| `throughput_settings.json` | Throughput sample sizes |
| `auto_protect_log.jsonl` | Recent automatic DNS/proxy actions |
| `config_watch_state.json` / `config_watch_events.jsonl` | Hosts file + system proxy change snapshots (Band C1) |
| `autoruns_settings.json` | Opt-in startup network-agent scan (Band C4) |

## Docs index

- [development.md](./development.md) — dev setup and verification commands
- [release.md](./release.md) — installer build and pre-push checklist
- [architecture.md](./architecture.md) — repo layout and module boundaries
- [platform-support.md](./platform-support.md) — what works on each OS
- [slowdown-factors.md](./slowdown-factors.md) — cross-platform slowdown research
- [slow-speed-triage.md](./slow-speed-triage.md) — symptom-to-signal triage matrix
- [product-plan-path-truth-2026-10.md](./product-plan-path-truth-2026-10.md) — Path Truth charter (**complete** at v1.8.1)
- [product-plan-cross-platform-capability-2026-10.md](./product-plan-cross-platform-capability-2026-10.md) — cross-platform expansion (**complete** in v1.9.0)
- [frontend-spec-mobile-companion.md](./frontend-spec-mobile-companion.md) — mobile companion UI (B1 designed; B2 implement)
- [../specs/backend/mobile-companion-design.md](../specs/backend/mobile-companion-design.md) — mobile companion backend (B1)
- [../specs/backend/hosts-proxy-timeline-design.md](../specs/backend/hosts-proxy-timeline-design.md) — hosts/proxy config watch (C1)
- [../specs/backend/path-break-classifier-design.md](../specs/backend/path-break-classifier-design.md) — Path Truth Band 1
- [../specs/backend/diagnostic-bundle-design.md](../specs/backend/diagnostic-bundle-design.md) — Path Truth Band 2
- [../specs/backend/integrity-board-design.md](../specs/backend/integrity-board-design.md) — Path Truth Band 3
- [../specs/backend/process-egress-design.md](../specs/backend/process-egress-design.md) — Path Truth Band 3E
- [../specs/backend/ai-explain-design.md](../specs/backend/ai-explain-design.md) — Path Truth Band 4 (4A–4C)
- [../specs/backend/egress-timeline-design.md](../specs/backend/egress-timeline-design.md) — Path Truth Band 5
- [../specs/backend/cli-check-design.md](../specs/backend/cli-check-design.md) — headless `check --json`
- [future-plan-security-integrity-2026-10.md](./future-plan-security-integrity-2026-10.md) — historical sketch (absorbed)
- [../specs/backend/local-first-improvement-roadmap.md](../specs/backend/local-first-improvement-roadmap.md) — L1–L6 local-first bands
- [../specs/backend/](../specs/backend/) — design specs
- [github-ci.md](./github-ci.md) — GitHub Actions CI and release publishing

## Principles

- Observe-first; assists are reversible and manual by default
- Local stack first (DNS, link, Windows caps, MTU); honest Tor/VPN copy; prove upstream pool blame
- Diagnosis ranks symptom shape and confidence; upstream pool proof does not auto-fix nodes
- No background throughput tests; on-demand samples only
- DNS integrity detects mismatches; it does not auto-change DNS unless Protect auto-apply is enabled
- Does not bypass corporate policy or claim guaranteed speed improvements
