# Diagnostic bundle export (Path Truth Band 2)

Status: **implemented** (core + Overview export + config timeline from local backups)  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §4, §8 order 2  
Depends on: [path-break-classifier-design.md](./path-break-classifier-design.md) (Band 1)

## Goal

Export a **redacted, AI-ready JSON bundle** from the latest `HealthReport` so novices/power users can:

1. Copy/save facts for offline review  
2. Paste into an external AI with a fixed prompt template  

**No cloud AI in this band.** Cloud opt-in is Band 4.

## Non-goals

- Sending data to any remote LLM  
- New probes beyond what health check already collected  
- Integrity board listeners/certs (Band 3 — placeholder `null` / listed in `not_checked`)  
- Silent elevation or repair from the bundle  

## Canonical owner

| Piece | Owner |
|-------|--------|
| Bundle builder | `network-core::diagnostic_bundle` |
| IPC | Tauri `export_diagnostic_bundle` |
| UI | Overview **Export diagnostic bundle** (copy JSON) |

## Schema (`DiagnosticBundle`)

```text
DiagnosticBundle
├── schema_version: "knottrace.diagnostic_bundle.v1"
├── exported_at: DateTime
├── app: { name, version? }
├── redaction: { include_ssid: bool, notes: Vec<string> }
├── os: string                     # std::env::consts::OS
├── environment_class: PathEnvironmentClass | null
├── path_break: summary fields     # active_path, break_hop, local_ok, confidence, title
├── score: { grade, score, summary }
├── diagnosis: { shape, confidence, primary_bottleneck, summary } | null
├── network_context: { kind, risk_level, captive_state, summary } | null
├── dns_integrity: { state, confidence, mismatch_count, summary } | null
├── proxy: { enabled, server_redacted }
├── tor: { detected, socks_reachable }
├── vpn_detected: bool
├── probe_table: Vec<ProbeRow>     # condensed path/hop outcomes
├── link_summary: string | null
├── upstream_pool: { claim, confidence, title } | null
├── recommendations: Vec<{ title, category, priority, action_id }>
├── integrity_flags: Vec<string>   # soft flags from existing signals only
├── config_timeline: Vec<string>   # from DNS/local-caps/MTU backups + auto-protect log when data_dir provided
├── not_checked: Vec<string>
└── ai_prompt_template: string     # fixed; instructs AI not to invent measurements
```

### ProbeRow

`path` (direct|system_proxy|vpn|tor|system), `stage` (dns|tcp|tls|http|reachability|local), `ok` bool, `detail` string

### Action IDs (stable)

| action_id | Meaning |
|-----------|---------|
| `open_dns_assist` | DNS Assist page |
| `open_connect` | Connect Assist |
| `open_network` | Network details |
| `open_protect` | Protect settings |
| `review_mtu` | MTU assist (manual) |
| `review_local_caps` | Windows local caps |
| `manual_out_of_scope` | No in-app action |

Map from `RecommendationCategory` in the builder.

## Redaction rules (v1)

| Field | Policy |
|-------|--------|
| Hostname | Omit |
| Interface names | Keep (useful, low sensitivity) |
| SSID | Omit unless `include_ssid: true` (UI default false; v1 always false) |
| DNS server IPs | Keep (needed for integrity) |
| Proxy server | Keep host:port (local/Clash typical) |
| Public egress IP | Keep if already in report (user exported intentionally) |
| Full HealthReport dump | **Never** — only curated fields |

## Orchestration

```text
build_diagnostic_bundle(report, DiagnosticBundleOptions { include_ssid }) -> DiagnosticBundle
```

Pure function; no I/O. Tauri returns the struct (serde JSON to UI). UI copies `JSON.stringify` to clipboard and optionally downloads `.json`.

## UI

- Overview diagnosis section: button **Export diagnostic JSON** (disabled until report exists)  
- Toast / inline: “Copied. Paste into an AI only if you accept sharing these fields.”  
- Show one-line list of redaction notes  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core diagnostic_bundle` |
| L2 | `cargo test -p network-core --lib` |
| L3 | Dev app: export copies JSON with `schema_version` and `ai_prompt_template` |

## Acceptance

- [x] Bundle builds from any successful `HealthReport`  
- [x] Hostname absent; SSID absent by default  
- [x] `not_checked` non-empty; includes Band 3 deferred items  
- [x] Recommendations carry `action_id`  
- [x] Overview export control present  
- [x] Docs + product plan Band 2 status updated  
- [x] L3 (runtime): bundle builder + export IPC present; frontend `tsc --noEmit` clean after smoke fix  
- [x] L3 (GUI): Diagnostic export panel visible; Copy JSON clicked in CDP smoke (2026-10-30)  



