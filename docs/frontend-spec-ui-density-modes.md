# Frontend spec — Quick vs Advanced density

Status: **implemented** (Bands U1–U5)  
Product plan: [product-plan-ui-density-modes-2026-11.md](./product-plan-ui-density-modes-2026-11.md)

## Goal

Progressive disclosure: **Quick** (default) for calm first session; **Advanced** for Path Truth depth. Persist in `localStorage` key `knottrace-ui-density` (`quick` | `advanced`).

## Chrome

- Canonical control: **Settings → Appearance → Detail level** (Quick | Advanced).
- No modal; immediate layout swap.
- Sidebar no longer hosts density/theme toggles (avoids chrome clutter).

## Overview — Quick hero composition

First viewport is **one StatusHero** (`emphasis="hero"`):

1. Plain-language grade headline (`Looking good` / `Needs attention` / …) + score badge  
2. Calm supporting line from **score.summary** when grade is good (not diagnosis “bottleneck” jargon)  
3. Inline latency strip (Internet / Gateway / DNS)  
4. At most **two** next-step buttons from `buildNextStepPlan`  

Below the hero: quieter Path + Integrity teasers, then Diagnostic export. No separate “What to do next” card in Quick.

## Overview — panel map

| Panel | Quick | Advanced |
|-------|-------|----------|
| StatusHero (hero composition + latency + CTAs) | Yes | StatusHero default + metric column |
| NextStepsPanel | In hero CTAs only | Yes (full card) |
| Path break (one-line teaser) | Yes (secondary) | Full PathBreakPanel |
| Integrity (count + level teaser → Protect) | Yes (secondary) | Full IntegrityBoardPanel |
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
| Link snapshot summary | Yes | No (score reasons under probe) |
| Probe results (latency strip) | Yes | Yes |
| LinkFactsPanel | Yes | Yes |
| LocalRepairPanel | Yes | Yes |
| Environment (full) | No | Yes |
| Stability / Local caps / MTU / Insights | No | Yes |
| Path break / Integrity / Egress / Proxy / Upstream / Tunnel / Throughput | No | Yes |

## DNS / Connect / Protect (Quick)

| Surface | Quick | Advanced |
|---------|-------|----------|
| AppHeader monitor poll/trigger copy | Hidden | Shown |
| FeaturePage page title | Embedded (badge only; AppHeader owns H1) | Same |
| DNS integrity settings editor | No | Yes |
| Connect Kernel/API FactList + API settings | Soft one-liner / hidden | Full |
| Protect Script risk scan section | Yes | Yes |
| Protect auto core toggles | Yes | Yes |
| Protect recovery + notifications + deeper opt-in scans | No (pointer to Advanced) | Yes |

See also: [frontend-spec-ux-pass-pages-2026-11.md](./frontend-spec-ux-pass-pages-2026-11.md).

## Copy rules

- Quick teasers: plain language; no badge piles.
- Never “secure”, “virus-free”, or “clean machine.”
- Integrity empty: “No indicators this check” (existing honesty).

## Non-goals

- CompanionShell redesign  
- New backend probes  
- Removing Advanced surfaces  
