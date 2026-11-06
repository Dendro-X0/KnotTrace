# Next-step ranking (B1) — design

**Date (UTC):** 2026-11-05  
**Status:** Implemented (L1 unit + desktop build green; L2 R1 optional on share)  
**Charter:** [product-plan-desktop-trust-path-truth-2026-11.md](../../docs/product-plan-desktop-trust-path-truth-2026-11.md) band **B1**  
**Owner:** `apps/desktop/src/lib/nextStepPlan.ts` (+ thin wrappers in `NextStepsPanel` / Overview Quick)  
**Out of scope:** B2 captive/bufferbloat hero copy · B3 tunnel Quick honesty · new probes · Rust recommendation engine changes

## Problem

`buildNextStepPlan` today ranks only by `slowdown_shape` and always emits 2–3 page CTAs. Gaps:

1. **`diagnosis.confidence` unused** — low-confidence / `general_degradation` still surfaces Protect + Network shotgun.
2. **Assist eligibility unused** — Connect / DNS appear even when `should_apply` is false or upstream-pool thrash is forbidden.
3. **Split ranking** — Overview Quick hero reads raw `plan.actions`; Advanced `NextStepsPanel` reorders for recurrence afterward. Same check → different primary CTA.
4. **Recurrence threshold skew** — UI mentions recurrence at count ≥ 2; reorder only at ≥ 3.

## Goal

One pure planner produces **1–3 ranked next steps** shared by Quick hero CTAs and the Advanced next-steps card:

- Prefer actions that match symptom shape + confidence ([slow-speed-triage.md](../../docs/slow-speed-triage.md)).
- Drop low-confidence clutter.
- Keep Assist destinations **only when eligible**.

## Input contract

Planner consumes a narrow snapshot (not full `CompanionState` UI methods):

| Field | Source |
|-------|--------|
| `shape` | `report.diagnosis.slowdown_shape` |
| `confidence` | `report.diagnosis.confidence` |
| `thrashForbidden` | upstream pool claim `upstream_pool_poor` \| `active_path_recurring` |
| `dnsAssistEligible` | `dnsRecommendation?.should_apply === true` |
| `connectAssistEligible` | `connectRecommendation?.should_apply === true` && !`thrashForbidden` |
| `integrityCount` | `report.integrity?.indicator_count ?? 0` |
| `recurrenceCount` | last ≤24 trends matching current shape (exclude `general_degradation`) |

## Ranking rules

### 1. Shape → ordered candidate pool

| Shape | Candidates (priority order) |
|-------|-----------------------------|
| `page_start` | DNS Assist (always as review) · Network · Protect only if integrity > 0 |
| `under_load_lag` | Network · Overview (benchmark / trends) |
| `partial_site_failure` | Network · Connect **if eligible** · else Overview claim when thrash forbidden |
| `restricted_network` | Protect · Network (no DNS/Connect as primary — B2 owns portal copy) |
| `tunnel_overhead` | Network · Connect **if eligible** · else Overview when thrash forbidden |
| `link_local_issue` | Network · Protect only if integrity > 0 |
| `general_degradation` | Network only |

### 2. Assist eligibility gate

- **DNS Assist CTA:** always allowed for `page_start` (review). For other shapes, include only when `dnsAssistEligible`.
- **Connect Assist CTA:** include only when `connectAssistEligible`. Never when thrash forbidden.
- Do not invent apply CTAs; labels stay “Open … Assist” (navigate only).

### 3. Confidence budget (truncate after filter)

| Confidence | Max actions |
|------------|-------------|
| `high` | 3 |
| `medium` | 2 |
| `low` | 1 |

If after filtering the pool is empty, emit a single `{ Open Network page → network }` fallback.

### 4. Title / summary

Keep existing shape titles for medium/high. For **`low` confidence** (any shape) or **`general_degradation`**:

- Title: `Review measured details first`
- Summary: soft language — hints are weak; inspect Network before changing DNS/proxy.

(B2/B3 may replace restricted / tunnel / bufferbloat copy later without changing this ranking owner.)

### 5. Recurrence reorder

When `recurrenceCount >= 2` and preferred page for the shape exists in the action list, move it to index 0 and mark `prioritized: true`. Preferred pages:

| Shape | Prefer |
|-------|--------|
| `page_start` | `dns` |
| `partial_site_failure` | `network` if thrash else `connect` if present else `network` |
| `restricted_network` | `protect` |
| `under_load_lag` | `network` |
| `tunnel_overhead` | `network` if thrash else `connect` if present else `network` |
| `link_local_issue` | `network` |

### 6. Quick hero surfacing (unchanged gate)

Overview Quick still hides next CTAs when `grade === "good"` unless integrity > 0 or path broken. Ranking owns order; hero gating owns visibility.

## Subtraction

- Remove duplicate recurrence logic from the panel — call the shared planner once.
- Do not add a second recommendation→CTA bridge from Rust for B1.

## Proof

| Layer | Command / check |
|-------|-----------------|
| L1 | `node --test apps/desktop/src/lib/nextStepPlan.test.mjs` (or package script) — confidence budget, assist gate, recurrence, Quick/Advanced same primary |
| L2 | `bash scripts/community-gate.sh` or `./scripts/reliability-gate.sh` after UI touch |
| L3 | Optional manual: low-confidence check → one Network CTA; page_start + DNS eligible → DNS first |

## Done when

- [x] Spec + frontend note committed
- [x] Single `buildNextStepPlan` used by Overview Quick and NextStepsPanel
- [x] Low confidence ≤1 action; Connect/DNS Assist only when eligible (DNS exception for `page_start`)
- [x] L1 tests green (`npm run test:unit` in `apps/desktop`); desktop `tsc` + `npm run build` green
- [x] L2 `bash scripts/reliability-gate.sh` green (2026-11-05)
