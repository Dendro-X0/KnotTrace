# Site / geo Quick playbook (P1) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **P1**  
**Owner:** `apps/desktop/src/lib/nextStepPlan.ts` + Overview Quick hero  
**Depends on:** B1 ranking · path-break classifier · proxy path report · investigation sibling  
**Out of scope:** new probes · auto rotation · geo-unblock guarantees · Advanced panel redesign

## Problem

Partial site failure collapses to one Quick line (“Compare the current path”) even when Path Truth already distinguishes local, DNS, proxy-only, and destination/policy shapes.

## Goal

When `diagnosis.slowdown_shape === "partial_site_failure"`:

1. Quick names the likely layer in plain language: **local / DNS / active path / destination or policy**.
2. One primary next step matches that layer.
3. Advanced Path / proxy / upstream panels stay the evidence home (unchanged).

## Input extensions

| Field | Source |
|-------|--------|
| `breakHop` | `path_break.break_hop` (`local` \| `dns` \| `tcp` \| `tls` \| `http` \| `unknown` \| `none` \| null) |
| `pathLocalOk` | raw `path_break.local_ok === true` (not tunnel-honesty `localOk`, which also requires `break_hop === none`) |
| `likelyProviderSide` | `proxy_path_report.likely_provider_side === true` |
| `proxyOnlyFailures` | `proxy_path_report.proxy_only_failure_count > 0` |
| `directAlsoFails` | `proxy_path_report.direct_failure_count > 0` |
| `environmentClass` | `path_break.environment_class` (optional reinforce) |

Existing: `localOk` (tunnel honesty), `activePath`, `thrashForbidden`, Connect/DNS eligibility.

## Layer classifier (`classifySiteFailureLayer`)

Only when `shape === "partial_site_failure"`; else `null`.

| Priority | Condition | Layer |
|---------:|-----------|--------|
| 1 | `breakHop === "dns"` | `dns` |
| 2 | `!pathLocalOk` **or** `breakHop === "local"` | `local` |
| 3 | `thrashForbidden` **or** `likelyProviderSide` **or** `proxyOnlyFailures` **or** (tunnel/proxy/Tor active **and** `pathLocalOk`) | `active_path` |
| 4 | else (incl. Direct + site fail, both paths fail, `filtered_region`) | `destination_or_policy` |

Notes:

- `directAlsoFails && !proxyOnlyFailures` must not classify as `active_path` via proxy-only; it falls through to destination/policy unless thrash/provider-side forced active_path.
- Tunnel active + partial shape → `active_path` (not B3 “slower expected”).

## Copy (i18n keys)

Low confidence still uses soft inspect-first copy (B1). Thrash remains a variant under `active_path`.

| Layer / variant | Title intent | Summary intent |
|-----------------|--------------|----------------|
| `local` | Local path looks broken first | Fix gateway / NIC / captive-adjacent local issues before blaming a site or node |
| `dns` | DNS looks involved | Resolver / integrity path first; do not thrash proxy nodes for DNS-shaped failures |
| `active_path` (normal) | Active path is impaired for some sites | Upstream pool / tunnel compare; at most one careful Connect Assist if eligible |
| `active_path` + thrash | Stop thrashing — pool looks poor | Keep existing thrash copy |
| `destination_or_policy` | Site or policy may be blocking | Destination, filter, or geo/policy — not a local repair; open Network for path evidence |

No “unblocked”, “safe”, or guaranteed Mbps language.

## Actions

| Layer | Primary pool (then confidence budget) |
|-------|----------------------------------------|
| `local` | Network; Protect only if integrity > 0. No Connect. DNS Assist only if eligible (after Network). |
| `dns` | DNS if Assist eligible, else Network. No Connect. |
| `active_path` | Network; Connect if eligible and not thrash; else Overview claim when thrash. |
| `destination_or_policy` | Network only. No Connect, no DNS-first. |

Recurrence: preferred page = first of that pool (`dns` → dns, `active_path` + Connect eligible → connect, else network).

## Priority vs other Quick honesty

`shapeCopy` / hero order:

1. Low confidence soft  
2. Captive (B2)  
3. Bufferbloat (B2)  
4. **Partial site playbook (P1)** when shape matches  
5. Tor SOCKS down / tunnel local-first / tunnel honesty (B3)  
6. Remaining shape switch

`candidatePool`: do **not** force Network-only tunnel override when partial playbook applies.

## Hero

When shape is `partial_site_failure` and captive/bufferbloat do not apply, hero uses layer keys (`nextSteps.hero.partialLocal` / `partialDns` / `partialActivePath` / `partialDestination`).

## Subtraction

- Do not add a fifth Quick CTA.  
- Do not duplicate PathBreakPanel text as a second Advanced card.  
- Do not change Rust `classify_path_break` in this band.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: each layer copy + actions; thrash variant; playbook beats tunnel honesty for partial shape; Connect omitted on local/dns/destination |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Manual optional: Direct OK · proxy partial · DNS hop · destination both-fail |

## Done when

- [x] Specs + frontend note  
- [x] Layer classifier + copy + actions wired  
- [x] Hero keys for four layers  
- [x] L1 unit green; L2 reliability-gate green  
