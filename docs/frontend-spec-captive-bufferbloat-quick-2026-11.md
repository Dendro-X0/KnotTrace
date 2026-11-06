# Frontend spec — Captive + bufferbloat Quick (B2)

**Date (UTC):** 2026-11-05  
**Status:** Implemented  
**API dependency:** existing `network_context.captive_portal`, `stability.bufferbloat`, diagnosis shapes — [captive-bufferbloat-quick-design.md](../specs/backend/captive-bufferbloat-quick-design.md)  
**Surfaces:** Overview Quick `StatusHero` summary + next CTAs · Advanced `NextStepsPanel` (same planner)

## Purpose

When the path problem is a hotspot portal or under-load queueing, Quick names it in plain language and points at the right one next step — without DNS-first clutter or “slow ISP” framing.

## Layout

No new panels. Behavior/copy only:

1. `quickHeroSummary` (extract to `nextStepPlan` or sibling helper) — shape-aware override before score summary.
2. `buildNextStepPlanFromInput` — captive/bufferbloat copy + captive action pool.
3. Overview Quick CTA gate — also open when captive or bufferbloat moderate/severe.

Advanced Network stability / context panels unchanged.

## Copy (user-visible)

### Captive confirmed
- Hero: “This Wi‑Fi needs sign-in before the internet works. Finish the portal in a browser, then re-check.”
- Next title/summary: design table (complete sign-in; no DNS/proxy yet).

### Captive suspected
- Hero: “Sign-in may be required on this network. Finish any portal (or switch networks), then re-check.”

### Bufferbloat moderate/severe / under_load_lag
- Hero: “Latency jumps under load (bufferbloat) — not a raw speed problem. Check router SQM/QoS if calls lag during downloads.”
- Next title: “Lag under load (bufferbloat)”

## Acceptance

- [x] Captive confirmed/suspected → first next-step page is `network`; no `dns` action
- [x] Bufferbloat Quick copy includes “bufferbloat” and does not say ISP is slow / guarantee Mbps
- [x] Hero prefers captive/bufferbloat lines over score summary when those signals fire
- [x] Advanced evidence panels untouched
