# Tunnel ∩ bufferbloat Quick line (P3) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **P3**  
**Owner:** `nextStepPlan` hero + next-step copy  
**Depends on:** B2 captive/bufferbloat · B3 tunnel honesty  
**Out of scope:** new probes · router SQM apply · Connect thrash

## Problem

Promoted bufferbloat hides tunnel honesty, so Quick pretends lag is only under-load queueing when a tunnel is also active.

## Goal

One combined Quick line when **both** are evidenced: tunnel overhead **and** under-load bufferbloat — not “only one.”

## Detector

```text
isTunnelBufferbloatOverlap(input):
  promoted bufferbloat (shape under_load_lag OR grade moderate|severe)
  AND NOT torSocksDown
  AND (
    isTunnelActivePath(activePath)
    OR shape === tunnel_overhead
  )
```

Local-not-OK still qualifies if tunnel is active (overlap can mention fix-local as secondary in summary only if needed — prefer one combined “both factors” line; Advanced Path/Network keep detail).

## Copy (en + zh-Hans)

| Key | Intent |
|-----|--------|
| `tunnelBufferbloatTitle` | Lag from tunnel **and** under-load queue |
| `tunnelBufferbloatSummary` | Overhead is expected on VPN/Tor/proxy; **also** latency jumps under load (bufferbloat). Open Network for tunnel compare + stability — local “speed” repair will not fix either alone. |
| `hero.tunnelBufferbloat` | Same story in one hero sentence |

No Mbps guarantees; no “safe.”

## Priority

`shapeCopy` / hero:

1. Low confidence soft  
2. Captive  
3. **Tunnel ∩ bufferbloat**  
4. Bufferbloat alone  
5. Partial site (P1)  
6. Tor SOCKS / tunnel local-first / tunnel honesty  
7. Remaining shapes  

## Actions

| Case | Pool |
|------|------|
| Overlap | Network primary; Overview benchmark secondary if budget ≥2. No Connect, no DNS. |

`candidatePool`: when overlap, do not apply tunnel Network-only early return after building bufferbloat pool — build overlap pool explicitly; skip DNS Assist append.

## Subtraction

- Do not show two hero lines.  
- Do not change Rust bufferbloat or tunnel classifiers.  
- Do not offer Connect as a bufferbloat fix on tunnels.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: overlap → combined keys + Network (±benchmark); no Connect; captive still wins; bufferbloat-only unchanged; tunnel-only honesty unchanged |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Optional: VPN + moderate bufferbloat screenshot |

## Done when

- [x] Specs + frontend note  
- [x] Detector + copy + hero wired  
- [x] L1 + L2 green  
