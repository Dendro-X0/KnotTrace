# Path evidence focus (U2) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **U2**  
**Owner:** `pathEvidenceFocus.ts` + Advanced `NetworkPage` Path section  
**Depends on:** existing HealthReport path panels  
**Out of scope:** Quick Network · new classifiers · Overview PathBreak CTA removal (optional keep)

## Problem

Path evidence is a flat wall of panels with no recommended entry and a redundant Run-check button.

## Goal

One **start here** recommendation; remaining Path panels quiet-by-default accordion; subtract duplicate Run-check on Network Path.

## Focus ranking (`recommendPathEvidenceFocus`)

First match wins:

| Priority | Condition | Focus id |
|---------:|-----------|----------|
| 1 | Tunnel active path (vpn/tor/system_proxy) **or** `tunnel_compare` with tor/vpn/proxy flags | `tunnel_compare` |
| 2 | Upstream claim ∈ impaired / recurring / pool_poor | `upstream_pool` |
| 3 | Proxy enabled **and** (`proxy_only_failure_count > 0` **or** `likely_provider_side`) | `proxy_path` |
| 4 | `path_break.break_hop` present and ≠ `none` | `path_break` |
| 5 | Egress claim ∈ `ip_changed` / `churning` / `unstable` | `egress_timeline` |
| 6 | Default | `path_break` |

`network_insights` is never the start-here focus (context only; always in accordion).

## UI

Path section header + start-here strip:

- Title key `pathFocus.startHere`  
- Body: localized reason for focus id  
- Recommended panel rendered **open** (full width of grid as today)  
- Other path panels each wrapped in `<details>` **closed** by default with summary = panel title  

Order in DOM: start-here → focused panel → accordion siblings (stable order: insights, path_break, proxy, upstream, tunnel, egress — skip focused).

## Subtraction

- Network Path: do **not** pass `onRunCheck` into `PathBreakPanel`.  
- Do not add a second Run-check in the start-here strip.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: ranking table cases |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Optional: Advanced Network Path screenshot |

## Done when

- [x] Specs + frontend note  
- [x] Mapper + Path section focus UI  
- [x] Duplicate Path Run-check removed on Network  
- [x] L1 + L2 green  
