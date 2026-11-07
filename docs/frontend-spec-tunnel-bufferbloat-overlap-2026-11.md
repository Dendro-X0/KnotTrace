# Frontend spec — Tunnel ∩ bufferbloat Quick (P3)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**API dependency:** existing path_break + stability.bufferbloat — [tunnel-bufferbloat-overlap-design.md](../specs/backend/tunnel-bufferbloat-overlap-design.md)  
**Surfaces:** Overview Quick hero · Advanced Next Steps (shared planner)

## Purpose

When tunnel path and bufferbloat are both evidenced, Quick uses one combined line instead of pretending it is only bufferbloat or only tunnel overhead.

## Layout

No new panels. Copy + ranking only in `nextStepPlan`.

## Keys

- `nextSteps.copy.tunnelBufferbloatTitle` / `tunnelBufferbloatSummary`
- `nextSteps.hero.tunnelBufferbloat`

en + zh-Hans.

## Acceptance

- [x] Tunnel + moderate/severe bufferbloat → combined title/hero  
- [x] No Connect in actions  
- [x] Captive still beats overlap  
- [x] Bufferbloat alone (Direct) unchanged  
- [x] Tunnel alone (no bufferbloat) still honesty keys  
