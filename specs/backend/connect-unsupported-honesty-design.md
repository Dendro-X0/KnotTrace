# Connect unsupported honesty (U1) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **U1**  
**Owner:** `ConnectPage` + `connectUnsupported.ts`  
**Depends on:** existing `ConnectRecommendation` discovery  
**Out of scope:** new discovery ports · auto Apply · changing Protect auto_apply_connect semantics

## Problem

No controllable proxy API still presents a Switch CTA (disabled) and under-explains that KnotTrace remains diagnosis-first for path/proxy.

## Goal

Unsupported Connect = calm diagnosis-only page: what is measured elsewhere; how to attach an API later; **no Apply/Switch control** until `canApply`.

## Detector

```text
isConnectUnsupported(rec):
  !rec || !rec.kernel
```

(Kernel missing is the unsupported case. Kernel present with empty groups is rare; treat as empty groups UI without Switch unless `should_apply && recommended_switch`.)

## UI rules

| Element | Unsupported | Supported + canApply | Supported + !canApply |
|---------|-------------|----------------------|------------------------|
| Footer Switch | **omit** | show enabled | **omit** (no disabled fake Apply) |
| Badge | Diagnosis only | Recommended / Connected | Connected / info |
| Body | EmptyState + measured list | groups list | groups / reason |
| Advanced API settings | keep (wire later) | keep | keep |

## Copy (en + zh-Hans)

| Key | Intent |
|-----|--------|
| `connectPage.unsupportedTitle` | No controllable proxy API |
| `connectPage.unsupportedSummary` | Connect Assist needs Mihomo/sing-box external-controller (or a saved API URL). KnotTrace still diagnoses path without switching nodes. |
| `connectPage.measuredTitle` | Still measured without Connect |
| `connectPage.measuredPath` | Path break / active path |
| `connectPage.measuredProxy` | Proxy path + upstream pool proof |
| `connectPage.measuredTunnel` | Tunnel compare (VPN/Tor/proxy vs Direct) |
| `connectPage.openNetwork` | optional hint text — open Network for evidence |
| `connectPage.badgeUnsupported` | Diagnosis only |

Do not claim unblock, Mbps boost, or “safe.”

## Helper

`apps/desktop/src/lib/connectUnsupported.ts`:

- `isConnectUnsupported(rec)`
- `shouldShowConnectSwitch(rec)` → kernel && should_apply && recommended_switch

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: unsupported → no switch; canApply → switch; copy keys stable |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Optional: open Connect with no Mihomo |

## Done when

- [x] Specs + frontend note  
- [x] No Switch footer when unsupported / !canApply  
- [x] Calm empty + measured list  
- [x] L1 + L2 green  
