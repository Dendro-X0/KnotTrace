# Frontend spec — Path evidence focus (U2)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**API dependency:** existing report fields — [path-evidence-focus-design.md](../specs/backend/path-evidence-focus-design.md)  
**Surfaces:** Network Advanced Path section only

## Purpose

Guide users to one Path panel first; keep the rest available but quiet; remove duplicate Run-check CTA.

## Layout

1. Section title `Path`  
2. Start-here callout (muted card / strip)  
3. Focused panel expanded  
4. Remaining panels as closed `<details>`  

## Acceptance

- [x] Tunnel-active report → start-here tunnel compare  
- [x] Upstream pool_poor → upstream pool  
- [x] Network PathBreak has no Run health check button  
- [x] en + zh-Hans start-here strings  
