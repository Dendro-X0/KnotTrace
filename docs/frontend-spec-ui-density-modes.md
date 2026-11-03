# Frontend spec — Quick vs Advanced density

Status: **implemented** (Bands U1–U5)  
Product plan: [product-plan-ui-density-modes-2026-11.md](./product-plan-ui-density-modes-2026-11.md)

## Goal

Progressive disclosure: **Quick** (default) for calm first session; **Advanced** for Path Truth depth. Persist in `localStorage` key `knottrace-ui-density` (`quick` | `advanced`).

## Chrome

- Sidebar footer (above/below theme): segmented **Quick | Advanced** control (same visual language as ThemeToggle).
- No modal; immediate layout swap.

## Overview — panel map

| Panel | Quick | Advanced |
|-------|-------|----------|
| StatusHero + latency metrics | Yes | Yes |
| NextStepsPanel | Yes (primary) | Yes |
| Path break (one-line teaser) | Yes | Full PathBreakPanel |
| Integrity (count + level teaser → Protect) | Yes | Full IntegrityBoardPanel |
| DiagnosticBundlePanel | Yes (help export) | Yes |
| NetworkDiagnosisPanel | No | Yes |
| EgressTimelinePanel | No | Yes |
| Local/Cloud explain | No | Yes |
| UpstreamPoolPanel | No | Yes |
| BenchmarkPanel | No | Yes |
| Trends | No | Yes |
| Recent checks | No | Yes |

## Network — panel map

| Panel | Quick | Advanced |
|-------|-------|----------|
| Probe results (latency strip) | Yes | Yes |
| LinkFactsPanel | Yes | Yes |
| LocalRepairPanel | Yes | Yes |
| Environment (full) | No | Yes |
| Stability / Local caps / MTU / Insights | No | Yes |
| Path break / Integrity / Egress / Proxy / Upstream / Tunnel / Throughput | No | Yes |

## Copy rules

- Quick teasers: plain language; no badge piles.
- Never “secure”, “virus-free”, or “clean machine.”
- Integrity empty: “No indicators this check” (existing honesty).

## Non-goals

- CompanionShell redesign  
- New backend probes  
- Removing Advanced surfaces  
