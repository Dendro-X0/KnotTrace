# Frontend spec — Empty & failure CTAs (A1)

**Status:** Shipped in tree  
**Plan:** [product-plan-desktop-trust-path-truth-2026-11.md](./product-plan-desktop-trust-path-truth-2026-11.md) band **A1**  
**Date:** 2026-11-05

## Goal

Each major empty / failure surface has **exactly one primary CTA**. Secondary links (GitHub, Releases) stay demoted. No competing “Run health check” + “Copy JSON” + “Open Protect” at the same weight.

## Surfaces (binding)

| Surface | Empty / failure | Primary CTA | Not primary |
|---------|-----------------|-------------|-------------|
| Check failure banner | Health check did not finish | **Retry check** | Issue / export advice (text only) |
| Update check failed | Cannot reach releases API | **Check again** | Open releases (ghost / link) |
| Diagnostic export | No report yet | **Run health check** | Copy JSON disabled/hidden until report |
| Integrity board / teaser | No report yet | **Run health check** | Protect navigation |
| Integrity board / teaser | Report, 0 indicators | *None* (honesty copy only) | Optional quiet “Open Protect” text control |
| Path break / next steps / trends (need check) | No report / insufficient history | **Run health check** | — |
| Script risk | Never scanned / 0 hits | **Run scan** (header) | Enable-on-check switch |
| Protect trust hero | No status / no report | **Run health check** | — |
| Protect trust hero | Report, 0 alerts | *None* (calm “All clear here”) | — |

## Copy rules

- Integrity empty with report: keep “Absence does not mean you are unobserved.”
- Script risk 0 hits: keep “does not mean the PC is malware-free.”
- Never “secure / clean / virus-free.”
- Prefer verb phrases: “Run health check”, “Retry check”, “Run scan”, “Check again”.

## Implementation notes

- Reuse `EmptyState` `action` slot with a single `Button`.
- Pass `onRunCheck` / use `state.runCheck` only where the parent already owns CompanionState.
- QuickIntegrityTeaser must not show “Clear this check” when `report` is null.

## Proof

- L2: `npx tsc --noEmit` + `npm run build` in `apps/desktop`
- L3: cold Overview Quick — export card shows Run health check; fail a check — banner Retry only; Protect with no report — hero CTA
