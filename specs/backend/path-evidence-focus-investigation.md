# Path evidence focus (U2) — investigation

**Date (UTC):** 2026-11-06  
**Status:** Complete  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **U2**  
**Owner (intended):** Advanced Network Path section (`NetworkPage`)  
**Out of scope:** new probes · redesign Link/Integrity/Throughput · Overview compact panels

## Symptom

Advanced Network **Path** dumps Path break, Proxy path, Upstream pool, Tunnel compare, Egress timeline, and Insights as equal cards. Users get no “start here,” and Path break empty state duplicates the global Run health check CTA.

## Evidence

| Panel | Signals |
|-------|---------|
| Path break | `path_break` hop / active path |
| Proxy path | `proxy_path_report` |
| Upstream pool | `upstream_pool.claim` |
| Tunnel compare | `tunnel_compare` + tunnel active path |
| Egress timeline | `egress_timeline.claim` |
| Insights | egress / context / recommendations |

Duplicate CTA: `PathBreakPanel` `onRunCheck` on Network Path while AppHeader already owns Run check.

## Recommended fix

1. Pure `recommendPathEvidenceFocus(report)` → one panel id + reason key.  
2. Path section: start-here line; recommended panel expanded; others quiet `<details>` closed.  
3. Drop Network Path `onRunCheck` on PathBreak (keep Overview compact if useful).

## Done when (investigation)

- [x] Duplicate CTA + equal-weight panels named  
- [x] Design can specify ranking  
