# Frontend spec — Next-step ranking (B1)

**Date (UTC):** 2026-11-05  
**Status:** Implemented  
**API dependency:** existing `HealthReport.diagnosis`, DNS/Connect `should_apply`, upstream pool claim — [next-step-ranking-design.md](../specs/backend/next-step-ranking-design.md)  
**Surfaces:** Overview Quick hero CTAs · Advanced `NextStepsPanel`

## Purpose

Make “what to do next” feel trustworthy: one ranked plan, fewer shotgun CTAs when confidence is low, Assist links only when eligible.

## Layout / chrome

No new panels. Behavior-only change inside existing Next Steps card and Quick hero primary/secondary actions.

### Quick (default)

- Hero primary/secondary = ranked `actions[0]` / `actions[1]` from shared planner (subject to existing good-grade gate).
- No third CTA in the hero.

### Advanced

- `NextStepsPanel` shows title, summary, confidence/shape badges, recurrence note, and the same ranked action buttons.
- “Prioritized” badge only when recurrence reorder moved an action to first.

## Copy rules

| Condition | Title / summary |
|-----------|-----------------|
| confidence `low` or shape `general_degradation` | Soft inspect-first copy (design §4) |
| other shapes medium/high | Keep current shape titles/summaries |

Do **not** add captive-portal or bufferbloat-specific hero lines here (B2). Do **not** add Tor “slower is expected” teaser (B3).

## Wiring

| Consumer | Import |
|----------|--------|
| `OverviewPage` | `buildNextStepPlan(state)` → `actions[0..1]` |
| `NextStepsPanel` | same function; do not re-rank locally |

Module: `apps/desktop/src/lib/nextStepPlan.ts`

## Acceptance

- [x] Low confidence → at most one button
- [x] Connect Assist absent when `!should_apply` or thrash forbidden
- [x] `page_start` still offers DNS Assist as review even if `!should_apply`
- [x] Quick hero primary matches Advanced first button for the same state
- [x] Empty/no-diagnosis empty state CTA unchanged (A1)
