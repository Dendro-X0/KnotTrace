# Tryout one-liner (U3) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **U3**  
**Owner:** COMMUNITY.md + Overview tip  
**Out of scope:** multi-step tour · forcing tip every launch after dismiss

## Goal

One sentence everywhere tryouts land: KnotTrace’s job is **path truth** (where the path breaks), **not** a speed boost.

## COMMUNITY

Insert immediately under the H1 / release line:

> **Job:** Path truth — tell whether the break is local, DNS, active path, or destination/policy. Not a speed booster.

Keep existing “What it is not” bullet; do not dilute with Mbps language.

## Overview tip (optional, ship it)

- Show once until dismissed (key `knottrace-path-truth-tip-dismissed`).  
- Quick Overview only (above bento).  
- Copy via i18n `overviewTip.pathTruth` + dismiss `overviewTip.dismiss`.  
- No CTA that implies Apply/boost.

Helper: `apps/desktop/src/lib/pathTruthTip.ts` — read/write dismiss + `shouldShowPathTruthTip()`.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: dismiss flip; show when unset |
| L2 | `bash scripts/reliability-gate.sh` |

## Done when

- [x] COMMUNITY one-liner  
- [x] Dismissible Overview tip  
- [x] L1 + L2 green  
