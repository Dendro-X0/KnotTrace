# Probe-language honesty (L1) — investigation

**Date (UTC):** 2026-11-06  
**Status:** Complete  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **L1**  
**Owner:** Quick hero / Advanced probe panels  
**Out of scope:** Full Rust diagnosis i18n · translating every hint title

## Symptom

zh-Hans UI chrome is localized, but `diagnosis.summary`, `score.summary`, path-break titles, and stability/probe narratives stay English — silent mixed UI.

## Evidence

| Surface | Language today |
|---------|----------------|
| Quick next-step / captive / bufferbloat / tunnel / partial keys | i18n |
| Quick hero fallback (`score` / `diagnosis` text) | English from Rust |
| Advanced Network diagnosis / path / stability panels | English from Rust |

## Decision

Charter allows localize **or** label. Do both lightly:

1. **Quick:** prefer localized next-step summary keys for shaped diagnoses before raw English text; when English text still shows in zh-Hans, show an “English measurement” note.  
2. **Advanced:** badge/note on diagnosis + path + stability probe panels in zh-Hans.

No claim that all probes are localized.

## Done when (investigation)

- [x] Mix mapped  
- [x] Dual light approach chosen  
