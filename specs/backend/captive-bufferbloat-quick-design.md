# Captive + bufferbloat Quick surfacing (B2) — design

**Date (UTC):** 2026-11-05  
**Status:** Implemented (L1 unit + desktop build green)  
**Charter:** [product-plan-desktop-trust-path-truth-2026-11.md](../../docs/product-plan-desktop-trust-path-truth-2026-11.md) band **B2**  
**Owner:** `apps/desktop/src/lib/nextStepPlan.ts` + Overview Quick hero summary  
**Depends on:** B1 [next-step-ranking-design.md](./next-step-ranking-design.md)  
**Out of scope:** B3 tunnel Quick honesty · portal bypass · router SQM remote config · new probes

## Problem

Diagnosis already classifies `restricted_network` (captive/guest) and `under_load_lag` (moderate/severe bufferbloat). Quick Overview still:

1. Prefers generic **score summary** in the hero — may read like “slow connection” instead of portal or under-load lag.
2. Leads captive users toward **Protect** (and B1 may still allow DNS on other shapes) rather than plain “finish sign-in / inspect Network.”
3. Names “latency under load” without saying **bufferbloat** or ruling out raw ISP Mbps.

## Goal

When those shapes fire, Quick hero + next steps use plain recovery / under-load copy. Advanced panels keep full evidence unchanged.

## Input extensions (on B1 snapshot)

| Field | Source |
|-------|--------|
| `captiveState` | `network_context.captive_portal.state` (`confirmed` \| `suspected` \| other → `none`) |
| `bufferbloatGrade` | `stability.bufferbloat.grade` (`moderate` \| `severe` \| else unused for B2 promote) |

## Captive / restricted rules

### Copy

| Condition | Title | Summary |
|-----------|-------|---------|
| `confirmed` captive | Complete Wi‑Fi sign-in | Open a browser, finish the hotspot portal, then re-check. Do not change DNS or proxy until the portal is done. |
| `suspected` captive | Likely hotspot sign-in | This network may require a login page. Finish sign-in (or switch networks), then re-check before assists. |
| `restricted_network` without captive (guest/public) | Verify access before tuning | Guest or public network — finish any portal, avoid sensitive traffic until trusted, then re-check. |

Low-confidence soft copy from B1 still wins when `confidence === "low"`.

### Actions

| Condition | Actions (then confidence budget) |
|-----------|----------------------------------|
| Captive confirmed/suspected | **Network only** as primary pool (portal/path evidence). No DNS, no Connect, no Protect as primary. |
| Restricted without captive | Protect → Network (trust posture), still no DNS/Connect |

Success criterion: Quick next steps **do not** lead with DNS apply/review for captive.

## Bufferbloat / under_load_lag rules

### Copy

| Condition | Title | Summary |
|-----------|-------|---------|
| shape `under_load_lag` or grade moderate/severe | Lag under load (bufferbloat) | Bandwidth is probably fine; latency jumps when something uploads or downloads. Check router SQM/QoS — this is not a raw “slow ISP” problem. |

### Actions

- Primary: Open Network page (stability / bufferbloat evidence in Advanced Network).
- Secondary (medium+ only): Save a benchmark — optional compare, not a speed-fix CTA.

### Hero promote

Even when score grade is `good`/`fair`, if shape is `under_load_lag` **or** bufferbloat grade is `moderate`/`severe`, Quick hero summary uses the bufferbloat line (not score Mbps-ish wording).

Same for captive confirmed/suspected: hero uses captive line ahead of score summary.

### CTA visibility

Extend Quick hero next-CTA gate: also surface when captive confirmed/suspected or bufferbloat moderate/severe (in addition to existing non-good grade / integrity / path-broken rules).

## Subtraction

- Do not duplicate copy in `QuickPathTeaser` for B2 (path teaser stays path-break owned; B3 may extend).
- Do not change Rust diagnosis classifiers.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Extend `nextStepPlan.test.ts` — captive → Network-only, no DNS; under_load copy names bufferbloat; hero helper unit if extracted |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Optional: captive + bufferbloat fixtures manually |

## Done when

- [x] Spec + frontend note
- [x] Captive Quick leads with portal/Network guidance, not DNS
- [x] Bufferbloat moderate/severe named in Quick hero + next steps without Mbps claim
- [x] L1 `npm run test:unit` (15 pass); desktop build green; L2 `bash scripts/reliability-gate.sh` green
