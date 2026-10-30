# Egress anomaly timeline (Path Truth Band 5)

Status: **implemented** (core + UI; history enrich on desktop monitor)  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §8 order 5  
Related: [public-network-egress-v1.3.md](./public-network-egress-v1.3.md), upstream pool history pattern

## Goal

Show a **local-first timeline** of public egress IP / stability anomalies from stored health checks — no new background heavy scans, no process↔egress spyware.

## Non-goals

- Continuous packet capture / netflow  
- Process correlation (still deferred)  
- Cloud upload of timeline  
- Guaranteeing “not monitored”  

## Canonical owner

| Piece | Owner |
|-------|--------|
| Classifier | `network-core::egress_timeline` |
| History enrich | Desktop `monitor` (same pattern as upstream_pool) |
| UI | Network **Egress timeline** + Overview compact |

## Data model

```text
EgressTimelineReport
├── claim: stable | ip_changed | churning | unstable | insufficient_history | none
├── confidence: high | medium | low
├── title / summary
├── current_ip: Option<string>
├── previous_ip: Option<string>
├── distinct_ips: u8
├── checks_considered: u8
├── events: Vec<{ at, kind, detail }>   # ≤12 newest-first
├── evidence / not_proven
└── action: string
```

### Claims

| Claim | Meaning |
|-------|---------|
| `insufficient_history` | &lt;2 checks with egress samples |
| `stable` | Same IP across window |
| `ip_changed` | Current IP ≠ previous sample (single change) |
| `churning` | ≥3 distinct IPs in window |
| `unstable` | Current egress probe low confidence / missing IP |
| `none` | No egress data on current check |

## Rules

Window: last 48 history reports (newest-first), same as upstream pool.

1. If current has no egress → `none`  
2. If `egress_unstable(current)` → prefer `unstable` (medium+)  
3. Collect non-empty primary IPs from series  
4. Distinct count ≥3 → `churning`  
5. Else if current IP ≠ immediately previous known IP → `ip_changed`  
6. Else if ≥2 samples same IP → `stable`  
7. Else → `insufficient_history`

Events: emit `ip_changed` when consecutive samples differ; `unstable_sample` when a check lacks IP / low confidence; cap 12.

## Orchestration

```text
# In run_health_check (snapshot):
report.egress_timeline = evaluate_egress_timeline_snapshot(&report);

# In desktop monitor after history load:
report.egress_timeline = evaluate_egress_timeline(&report, &history);
```

Rebuild recommendations/diagnosis after enrich (match upstream_pool).

## Opt-in posture

v1 is **observe-only** from existing history (no extra probes). “Opt-in” for Band 5 means: no new always-on scanners; timeline uses checks the user/monitor already ran. Future: toggle to hide panel — not required for first ship.

## UI

- Network: panel with claim badge, current/previous IP, recent events  
- Overview: compact title + summary when claim ≠ stable/insufficient  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core egress_timeline` |
| L2 | full lib tests |
| L3 | runtime probe + history enrich path (GUI if RC.EXE) |

## Acceptance

- [x] Snapshot + history evaluators  
- [x] Desktop enrich wired  
- [x] Unit fixtures: stable / changed / churning / unstable  
- [x] UI panel  
- [x] Docs + product plan Band 5  
- [ ] L3 GUI click-smoke when RC.EXE available  
