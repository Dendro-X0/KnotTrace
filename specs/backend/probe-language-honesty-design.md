# Probe-language honesty (L1) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **L1**  
**Owner:** `probeLanguage.ts` + Overview / Network panels  
**Out of scope:** translating Rust `build_summary` / hint catalogs

## Goal

No silent mixed UI in zh-Hans: either localized Quick shape copy, or an explicit English-measurement label on remaining probe text.

## Helper

`apps/desktop/src/lib/probeLanguage.ts`:

- `needsEnglishProbeLabel(locale)` → `locale === "zh-Hans"`  
- i18n key `probeLanguage.englishMeasurements` (en still defined for completeness; UI shows label only when needed)

## Quick

1. In `buildQuickHeroSummary`, after special-case heroes, if `slowdown_shape` is set and ≠ `general_degradation`, prefer `buildNextStepPlan(state).summaryKey` as `{ kind: "key" }` before raw score/diagnosis English.  
2. Overview Quick: when locale needs label **and** hero is `{ kind: "text" }`, render `EnglishProbeNote` under the hero summary.

## Advanced

When `needsEnglishProbeLabel`:

- `NetworkDiagnosisPanel` — badge next to title  
- `StabilityProbesPanel` — badge next to title  
- `PathBreakPanel` — badge next to title (non-empty)  
- Network Path start-here strip — one-line note  

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: needsEnglishProbeLabel; hero prefers shape key over English when shape set |
| L2 | `bash scripts/reliability-gate.sh` |

## Done when

- [x] Specs + frontend note  
- [x] Helper + Quick preference + labels  
- [x] L1 + L2 green  
