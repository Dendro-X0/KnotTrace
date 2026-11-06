# Frontend spec — Settings & chrome consistency (A3)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**Charter:** [product-plan-desktop-trust-path-truth-2026-11.md](./product-plan-desktop-trust-path-truth-2026-11.md) band **A3**  
**Depends on:** [frontend-spec-settings-page-2026-11.md](./frontend-spec-settings-page-2026-11.md), B1/B2 next-step copy

## Audit (current tree)

| Control | Settings | Sidebar | AppHeader Quick | Verdict |
|---------|----------|---------|-----------------|---------|
| Theme | Yes | No | No | OK |
| Density | Yes | No | No | OK |
| Language | Yes | No | No | OK |
| Background monitor | Yes | No | **Yes (Switch)** | Duplicate — quiet Quick |
| Protect opt-ins | Link to Protect | No | No | OK |
| Run health check | No | No | Always + Overview hero when no report | Competing on Overview Quick |

## Changes

1. **Quick AppHeader** — remove monitor Switch; Settings Monitoring remains canonical. Advanced keeps header Switch + badge/detail.
2. **Overview Quick** — hero primary is next-step only; do not duplicate “Run health check” when header already owns it (empty report → header CTA only).
3. **i18n** — add `nextSteps.*` (+ header monitor detail) for B1/B2 planner copy and NextStepsPanel chrome; `zh-Hans` translations. Backend diagnosis/score strings stay English (existing i18n non-goal).

## Non-goals

- Moving Protect toggles into Settings body  
- Translating all Network/Protect probe copy  
- B3 tunnel honesty  

## Acceptance

- [x] Quick header has no monitor toggle  
- [x] Overview Quick no-report state has a single Run check (header)  
- [x] Next-step titles/summaries/actions resolve via `t()` with en + zh-Hans  
- [x] Theme/density/language still only on Settings  
