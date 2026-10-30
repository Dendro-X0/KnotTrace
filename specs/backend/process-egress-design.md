# Process ↔ egress correlation (Path Truth Band 3E)

Status: **implemented**  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §6 “Opt-in later”  
Depends on: Band 3 integrity board, egress probe

## Goal

When the user **opts in**, sample which local processes currently hold established TCP sessions toward the **observed public egress IP** and/or **system proxy host** — as **indicators only**.

Canonical honesty still applies: absence of hits does not mean unobserved; hits do not mean malware.

## Non-goals

- Continuous packet capture / netflow  
- Full process tree / DLL injection detection  
- Auto-kill or quarantine  
- Default-on scanning  
- Guaranteeing attribution of all egress (UDP, QUIC, kernel, etc. omitted)

## Canonical owner

| Piece | Owner |
|-------|--------|
| Settings | `network-core::process_egress` (`process_egress_settings.json`) |
| Scan + indicators | `network-core::integrity_board` / `process_egress` |
| Opt-in UI | Protect page toggle |
| Apply on check | Desktop `monitor` after health check when enabled |

## Settings

```text
ProcessEgressSettings
├── enabled: bool          # default false
└── consent_ack: bool      # must be true to enable
```

Consent copy (short): KnotTrace will briefly query OS connection tables and process names for sessions matching your current egress/proxy. Nothing is uploaded. You can turn this off anytime.

## Data model

```text
ProcessEgressSnapshot
├── scanned: bool
├── enabled: bool
├── source: string
├── matched_targets: Vec<string>   # egress IP / proxy host matched
├── hits: Vec<{ process_name, pid?, remote, count }>  # ≤12
└── summary: string

IntegrityIndicatorKind += process_egress_hint
```

Attach `process_egress: ProcessEgressSnapshot | null` on `IntegrityBoardReport`.

## Scan rules

1. If settings `enabled` is false → skip; keep “Process ↔ egress correlation” in `not_checked`.
2. Collect match targets: `egress.primary_ip` if present; proxy host from `environment.proxy.server` if proxy enabled.
3. If no targets → `scanned: true`, empty hits, summary “No egress/proxy target to match.”
4. Query established TCP remotes, filter to targets, group by PID, resolve process name (cap 12 hits):

| OS | Method |
|----|--------|
| Windows | `Get-NetTCPConnection -State Established` |
| Linux | `ss -H -tnip` |
| macOS | `lsof -nP -iTCP -sTCP:ESTABLISHED` |

5. If any hits → push indicator `process_egress_hint` (info): title “Processes with sessions toward current egress/proxy”; message lists top names; evidence = summary counts.
6. Failures → `scanned: false`, no fake clean bill; leave honesty note if useful.

## Orchestration

```text
# Snapshot assess always omits process scan (CLI default).
report.integrity = assess_integrity_board(&report);

# Desktop after check, if settings.enabled:
enrich_process_egress(&mut report, &settings);
```

Re-finalize indicator count/level/title after pushes. Drop process line from `not_checked` when `process_egress.scanned` (attempt completed) **or** when enabled+scanned true.

## UI

- Protect: opt-in toggle + consent checkbox (mirrors AI explain posture, shorter copy)
- Network Integrity panel: show snapshot summary when scanned

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core process_egress` |
| L2 | lib tests |
| L3 | default settings → process still in `not_checked`; with fixture enrich → indicator + snapshot |

## Acceptance

- [x] Default off  
- [x] Windows scan + indicator fixtures  
- [x] Protect toggle + IPC  
- [x] Docs / product plan polish note  
