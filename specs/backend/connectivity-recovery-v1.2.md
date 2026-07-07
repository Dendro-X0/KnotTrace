# Connectivity recovery (v1.2)

## Problem

Users on proxy or multi-hop paths (Clash, Mihomo, Tor, VPN chains) often lose access to sites when:

- A proxy node has an unstable or blocked egress IP
- DNS is poisoned or hijacked on the path (proxy scripts, captive portals, malware)
- The system resolver returns answers that differ from trusted resolvers

Native connectivity may look "up" (TCP to 1.1.1.1 works) while HTTPS to real sites fails.

## Principles

1. **Observe first** — classify before acting; log what changed.
2. **Reversible assists only** — DNS backup + proxy group switch; no permanent firewall edits.
3. **Rate-limited automation** — 10-minute auto-protect cooldown (existing).
4. **No bypass** — do not defeat corporate policy, CAPTCHA walls, or geo-blocks.
5. **Windows DNS apply** — other platforms recommend until `dns_platform` expands.

## v1.2.0 (this release)

| Feature | Behavior |
|---------|----------|
| **Site reachability probe** | HTTPS HEAD to a small set of verification domains (shared with DNS integrity list, max 4 per check). Uses system proxy when enabled. |
| **Diagnosis hints** | "Sites unreachable via current path" when probe failures cluster with proxy on. |
| **Auto-protect: DNS integrity** | When integrity is caution/suspicious (medium+ confidence), auto-apply trusted DNS if enabled (even on "trusted" networks). |
| **Auto-protect: proxy path** | When site reachability fails with proxy on, auto-switch Clash node if Connect Assist recommends one. |
| **Protect toggle** | `auto_recover_site_access` — default on; limits automatic node switching to reachability failures. |

## v1.2.x follow-ups

| Item | Notes |
|------|-------|
| User watchlist domains | Extend DNS integrity editor; reachability uses same list |
| DIRECT / rule fallback | Suggest switching selector to DIRECT when all nodes fail (manual confirm first) |
| Linux/macOS DNS apply | `resolvectl` / NetworkSetup |
| Captive portal hint | Detect generate_204 vs login page |
| Recovery audit log | JSONL of auto actions with timestamp + rollback |

## Exit criteria (v1.2.0)

- Health check includes `site_reachability` when integrity domains configured
- Protect auto-run triggers on DNS integrity OR site failures (not only untrusted Wi-Fi)
- Diagnosis surfaces site-access bottleneck category
- Tests for trigger logic and reachability summarization
