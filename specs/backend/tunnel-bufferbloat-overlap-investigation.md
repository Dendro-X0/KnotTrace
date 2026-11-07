# Tunnel ∩ bufferbloat Quick line (P3) — investigation

**Date (UTC):** 2026-11-06  
**Status:** Complete  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **P3**  
**Owner (intended):** `apps/desktop/src/lib/nextStepPlan.ts`  
**Out of scope:** new probes · SQM remote config · Mbps claims · Connect “speed up tunnel”

## Symptom

When VPN/Tor/proxy is active **and** bufferbloat is moderate/severe, Quick hero / next steps promote **only** bufferbloat (B2). Tunnel honesty (B3) never runs. Users cannot tell whether lag is path overhead, under-load queueing, or both.

## Evidence already present

| Signal | Source |
|--------|--------|
| Tunnel active | `path_break.active_path` ∈ vpn/tor/system_proxy |
| Local OK (honesty) | `localOk` / path_break |
| Bufferbloat | `stability.bufferbloat.grade` moderate\|severe or shape `under_load_lag` |
| Shape | may be `tunnel_overhead` or `under_load_lag` depending on diagnosis priority |

## Gap vs charter

§7: “Tunnel + bufferbloat both evidenced → Quick does not pretend it’s only one.”

Today: bufferbloat branch wins before tunnel honesty in `shapeCopy` and `buildQuickHeroSummary`.

## Root cause

B2 priority is absolute for any promoted bufferbloat; B3 has no intersection case.

## Recommended fix

1. Detect overlap: tunnel active (or shape `tunnel_overhead`) **and** promoted bufferbloat; SOCKS-down stays its own case.
2. Combined i18n title/summary/hero naming **both** overhead and under-load queue.
3. Priority: captive → **overlap** → bufferbloat-only → … → tunnel honesty.
4. Actions: Network (+ optional Overview benchmark); no Connect/DNS.

## Done when (investigation)

- [x] Conflict mapped  
- [x] Design can specify copy + priority  
