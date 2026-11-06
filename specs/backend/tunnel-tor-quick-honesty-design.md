# Tunnel / Tor Quick honesty (B3) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + desktop build green)  
**Charter:** [product-plan-desktop-trust-path-truth-2026-11.md](../../docs/product-plan-desktop-trust-path-truth-2026-11.md) band **B3**  
**Owner:** `QuickPathTeaser` + `nextStepPlan` (hero / next-step honesty)  
**Depends on:** B1 ranking, B2 captive/bufferbloat priority, existing `tunnel_compare` / `path_break`  
**Out of scope:** Tor control · auto node rotation · Mbps claims · new probes

## Problem

When VPN / Tor / system proxy is the active path and the local stack looks OK, Quick can still feel like “something is broken — repair locally.” Path teaser only echoes `path_break` title/summary; next-step `tunnel_overhead` can still offer Connect as if a node switch will “speed up” privacy paths.

## Goal

When a tunnel path is active:

1. **Path teaser** shows one plain **expectation** line: slower/anonymity trade-off vs local broken.
2. **Next steps / hero** (when this shape wins over captive/bufferbloat) do **not** imply local repair or thrash will accelerate Tor/VPN.
3. Advanced Tunnel compare panel stays the evidence home (unchanged).

## Signals

| Signal | Source |
|--------|--------|
| Tunnel active | `path_break.active_path` ∈ `vpn_tunnel` \| `tor_socks` \| `system_proxy` |
| Local OK | `path_break.local_ok === true` and `break_hop === "none"` |
| SOCKS down | `tunnel_compare.tor_detected && !tor_socks_reachable` |
| Shape | `diagnosis.slowdown_shape === "tunnel_overhead"` (optional reinforce) |

## Path teaser rules

| Condition | Expectation line |
|-----------|------------------|
| Tor SOCKS down | Prefer `tunnel_compare.expectation` (client bootstrap — not ISP down) |
| Tunnel active + local OK | Prefer short `tunnel_compare.expectation` if present; else i18n by path kind (`nextSteps.hero.tunnelExpected` / Tor / VPN / proxy variants) |
| Tunnel active + local NOT OK | Keep path-break title/summary; append i18n note: fix local first; tunnel overhead is secondary |
| Direct only | Unchanged (no expectation line) |

Do not invent Mbps numbers in the teaser; if expectation includes sampled latency from backend, showing that text is OK (measured fact).

## Next-step / hero rules

Priority (unchanged): captive → bufferbloat → **tunnel honesty** → score/diagnosis.

When tunnel honesty applies (`tunnel active + local OK`, or shape `tunnel_overhead` without thrash):

| Field | Behavior |
|-------|----------|
| Title/summary keys | `tunnelHonesty*` — slower/anonymity expected; local repair will not speed this path |
| Actions | **Network only** (tunnel compare / path evidence). No Connect, no DNS, no benchmark “speed” CTA |
| Thrash forbidden | Keep existing thrash copy + Network/Overview claim (still no Connect) |

SOCKS-down case: Network primary; copy points at Tor client, not NIC repair.

## Subtraction

- Do not add a second hero CTA that says “fix DNS” or “save benchmark” for tunnel-overhead honesty.
- Do not change Rust classifiers; reuse `expectation` string when useful.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit tests: tunnel+local OK → Network-only + honesty keys; path teaser helper; Connect omitted |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Optional: Tor SOCKS up / VPN active Quick screenshot |

## Done when

- [x] Spec + frontend note
- [x] Quick Path teaser shows expectation when tunnel active
- [x] Tunnel+local OK does not offer Connect/DNS/benchmark as next steps
- [x] L1 `npm run test:unit` (21 pass); desktop build green; L2 `bash scripts/reliability-gate.sh` green
