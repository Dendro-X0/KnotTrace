# Connect unsupported honesty (U1) — investigation

**Date (UTC):** 2026-11-06  
**Status:** Complete  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **U1**  
**Owner (intended):** `ConnectPage` + small pure helper  
**Out of scope:** inventing a controller · auto-install Mihomo · Apply without API

## Symptom

With no Mihomo/sing-box Clash API, Connect still shows a **disabled** “Switch to recommended node” footer and an empty state that only says “enable external-controller.” Reads like a broken Apply, not a calm diagnosis-only surface.

## Evidence

| Signal | Source |
|--------|--------|
| No kernel | `connectRecommendation.kernel == null` |
| No groups | `groups.length === 0` |
| `should_apply` | false when unsupported |
| Reason string | backend discovery message (English) |
| Footer | always rendered in `ConnectPage` |

Path / proxy / tunnel evidence still exists on Overview/Network without Connect API.

## Gap vs charter

“If no controllable proxy API: calm empty state — diagnosis-only; what *is* measured; no fake Apply.”

Today: fake-looking Apply + incomplete empty copy.

## Recommended fix

1. Omit footer Switch unless `canApply`.  
2. Unsupported empty: diagnosis-only title/summary + short “still measured” bullets + optional Advanced API settings.  
3. Badge: “Diagnosis only” when no kernel.  
4. Pure helper for unsupported detection + copy keys (unit-testable).

## Done when (investigation)

- [x] Dead Apply path identified  
- [x] Design can specify UI rules  
