# Frontend spec — Site / geo Quick playbook (P1)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**API dependency:** existing `path_break`, `proxy_path_report`, `upstream_pool`, diagnosis shape — [site-geo-quick-playbook-design.md](../specs/backend/site-geo-quick-playbook-design.md)  
**Surfaces:** Overview Quick hero · Advanced `NextStepsPanel` (shared planner)

## Purpose

On `partial_site_failure`, Quick names **local / DNS / active path / destination-or-policy** and offers one primary next step. Advanced evidence panels unchanged.

## Layout / chrome

No new panels. Behavior + copy only inside shared `nextStepPlan` and existing hero / Next Steps.

## Copy keys (`nextSteps.copy` / `nextSteps.hero`)

| Key | When |
|-----|------|
| `partialLocalTitle` / `partialLocalSummary` | layer `local` |
| `partialDnsTitle` / `partialDnsSummary` | layer `dns` |
| `partialTitle` / `partialSummary` | layer `active_path` (reuse; tune if needed) |
| `partialThrashTitle` / `partialThrashSummary` | `active_path` + thrash |
| `partialDestinationTitle` / `partialDestinationSummary` | layer `destination_or_policy` |
| `hero.partialLocal` / `partialDns` / `partialActivePath` / `partialDestination` | Quick hero |

en + zh-Hans required.

## Wiring

| Consumer | Behavior |
|----------|----------|
| `buildNextStepPlanFromInput` | classify layer → copy + actions |
| `buildQuickHeroSummary` | after captive/bufferbloat, before tunnel honesty |
| `NextStepsPanel` / Overview | unchanged imports |

## Acceptance

- [x] Partial + DNS hop → DNS-layer copy; no Connect  
- [x] Partial + `!pathLocalOk` → local-layer copy; Network primary  
- [x] Partial + proxy-only / thrash → active-path (thrash variant when forbidden)  
- [x] Partial + Direct / both-fail → destination-or-policy; Network only  
- [x] Partial + tunnel active → playbook, not B3 “slower expected”  
- [x] Advanced PathBreakPanel still shows full evidence  
