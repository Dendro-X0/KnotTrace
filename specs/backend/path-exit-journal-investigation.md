# Path / exit journal (P2) — investigation

**Date (UTC):** 2026-11-06  
**Status:** Complete  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **P2**  
**Owner (intended):** history trend projection + Overview Quick/Advanced UI  
**Out of scope:** new probe engines · continuous capture · process↔egress · cloud sync of journal

## Symptom

Users cannot see a compact “what path / exit looked like recently” without reading raw trends charts or the egress-claim card alone. Charter asks for recent: **active path**, **egress IP**, **slowdown shape**, **score grade**.

## What history already stores

Full `HealthReport` JSON is in `history.db`. `HistoryTrendPoint` (via `report_to_trend_point`) already projects:

| Field | Present? |
|-------|----------|
| `grade` / `score` | Yes |
| `slowdown_shape` | Yes |
| `egress_ip` | Yes |
| `proxy_enabled`, proxy-only counts, `upstream_claim` | Yes |
| `active_path` (path_break) | **No** — gap |

Egress timeline is a **claim summarizer** (stable / churning / …), not a row journal of path+shape+grade.

## Gap

Without `active_path` on trend points, a journal would invent path from `proxy_enabled` alone and miss VPN/Tor labeling that Path Truth already computed per check.

## Root cause

Trend projection never lifted `path_break.active_path` when Path Truth shipped.

## Recommended fix

1. Add optional `active_path` to `HistoryTrendPoint` from `report.path_break.active_path` (serde skip if none; old rows without path_break stay null).
2. Pure frontend mapper builds ≤N newest journal rows (path, egress, shape, grade, time).
3. Overview: Quick compact teaser + Advanced card; reuse trends already loaded (`get_history_trends`).
4. No new SQLite schema — projection-only on existing JSON.

## Done when (investigation)

- [x] Fields mapped; single gap (`active_path`) named  
- [x] No new probe required  
- [x] Design can specify UI + proof  
