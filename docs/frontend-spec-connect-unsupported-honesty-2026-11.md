# Frontend spec — Connect unsupported honesty (U1)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**API dependency:** existing `ConnectRecommendation` — [connect-unsupported-honesty-design.md](../specs/backend/connect-unsupported-honesty-design.md)  
**Surfaces:** Connect page (Quick + Advanced)

## Purpose

When no controllable proxy API is present, Connect is diagnosis-only: explain what is still measured, how to attach an API, and never show a dead Switch/Apply control.

## Layout

- Unsupported: EmptyState (title + summary) + short measured list; Advanced keeps API settings accordion.  
- Footer: only when `shouldShowConnectSwitch`.  
- Quick: same empty honesty; hide API accordion (unchanged).

## Acceptance

- [x] No kernel → no Switch button in DOM footer  
- [x] Empty copy names diagnosis-only + measured path/proxy/tunnel  
- [x] Kernel + recommended switch → Switch still available  
- [x] en + zh-Hans keys  
