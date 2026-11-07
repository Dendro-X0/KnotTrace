# Site / geo Quick playbook (P1) — investigation

**Date (UTC):** 2026-11-06  
**Status:** Complete  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **P1**  
**Owner (intended):** `apps/desktop/src/lib/nextStepPlan.ts`  
**Out of scope:** new reachability probes · auto node rotation · “unblock this country” claims · Rust classifier changes

## Symptom

When diagnosis shape is `partial_site_failure`, Quick still says generic “Compare the current path” / pool thrash copy. Path Truth already computes richer evidence (`path_break.break_hop`, `proxy_path_report`, upstream claim), but the Quick next-step layer does not name **local / DNS / active path / destination-or-policy** in plain language.

## Evidence already present (no new engines)

| Signal | Source | Use for layer |
|--------|--------|----------------|
| Local OK / break hop | `path_break.local_ok`, `break_hop` | `local`, `dns` |
| Proxy-only failures | `proxy_path_report.proxy_only_failure_count`, `likely_provider_side` | `active_path` |
| Thrash / pool | `upstream_pool.claim` ∈ poor / recurring | `active_path` + thrash copy |
| Active path kind | `path_break.active_path` | tunnel → prefer active-path playbook over “slower expected” when shape is partial |
| Direct also fails | `proxy_path_report.direct_failure_count > 0` with failures | leans `destination_or_policy` when not proxy-only |
| Site reach failures on Direct | shape + Direct active + hop tls/http/unknown | `destination_or_policy` |
| Environment class | `path_break.environment_class === filtered_region` | soft reinforce destination/policy |

## Gap vs charter success criterion

Charter §7: “On partial site failure, Quick names the likely layer in plain language (not only `partial_site_failure` badge).”

Today: badge/shape + one generic partial title. Advanced Path break / proxy panels already hold evidence — keep them; Quick needs a playbook mapping.

## Priority conflict with B3

`shapeCopy` currently applies tunnel honesty **before** the `partial_site_failure` switch. A VPN/Tor path with partial site failures can therefore get “Slower is expected” instead of a site/geo layer verdict.

**Decision for P1:** when `slowdown_shape === "partial_site_failure"`, site/geo playbook wins over tunnel honesty (captive + bufferbloat still win first). Tunnel honesty remains for `tunnel_overhead` / tunnel-active without this shape.

## Root cause

Frontend ranking treats partial sites as one copy bucket + Connect eligibility, not as a four-way layer derived from path-break / proxy-path signals already on the report.

## Recommended fix (design follows)

1. Extend `NextStepInput` with break hop + proxy-path booleans (extract-only).
2. Pure `classifySiteFailureLayer` → `local | dns | active_path | destination_or_policy`.
3. Layer-specific i18n title/summary + primary action rules; thrash remains a variant of `active_path`.
4. Quick hero key when shape is partial and higher-priority heroes do not apply.
5. Unit tests per layer; no Rust changes.

## Done when (investigation)

- [x] Signals mapped without new probes
- [x] Priority vs B3 decided
- [x] Design doc can specify copy + actions
