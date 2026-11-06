# Frontend spec — Advanced Network navigation (A2)

**Status:** Shipped in tree  
**Plan:** [product-plan-desktop-trust-path-truth-2026-11.md](./product-plan-desktop-trust-path-truth-2026-11.md) band **A2**  
**Date:** 2026-11-05

## Goal

Advanced **Network** is jumpable by job, not a single vertical avalanche. **Quick** layout stays unchanged.

## Sections

| Id | Label | Panels |
|----|-------|--------|
| `network-link` | Link & local | Environment, probes, stability, link facts, local repair, local caps, MTU |
| `network-path` | Path | Insights, path break, proxy path, upstream pool, tunnel compare, egress timeline |
| `network-integrity` | Integrity | Integrity board |
| `network-throughput` | Throughput | Throughput sample |

## Chrome

- Sticky horizontal nav (pills) above the Advanced grid; click scrolls the matching `<section>` into view (`scrollIntoView({ behavior: "smooth", block: "start" })`).
- Each section has a quiet uppercase label + the existing panels in the same 1→2 column grid.
- No new probes, no Quick changes, no tabs that hide content by default (anchors only — all panels remain mounted).

## Non-goals

- Sidebar rename (“Network” vs path language) — later polish  
- Collapsing sections closed by default  
- CompanionShell Network surface  

## Proof

- L2: `npx tsc --noEmit` + `npm run build` in `apps/desktop`
- L3: Advanced → Network → click Path / Integrity pills; Quick Network viewport unchanged
