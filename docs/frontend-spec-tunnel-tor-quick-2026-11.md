# Frontend spec — Tunnel / Tor Quick honesty (B3)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**API dependency:** `path_break`, `tunnel_compare.expectation` — [tunnel-tor-quick-honesty-design.md](../specs/backend/tunnel-tor-quick-honesty-design.md)  
**Surfaces:** Overview Quick Path teaser · Quick hero / next steps (planner)

## Purpose

When Tor/VPN/proxy is active and local looks OK, Quick says **slower is expected** — not “repair your PC to go faster.”

## Layout

1. **QuickPathTeaser** — existing title/summary; add one muted expectation line when tunnel-active rules fire. Pass `tunnelCompare` from Overview.
2. **nextStepPlan** — honesty copy + Network-only actions; hero key after captive/bufferbloat.
3. Advanced Tunnel compare unchanged.

## Copy keys (`nextSteps.*`)

| Key | Use |
|-----|-----|
| `hero.tunnelExpected` | Generic VPN/proxy/Tor slower-expected hero |
| `hero.torSocksDown` | Tor client/bootstrap, not ISP |
| `copy.tunnelHonestyTitle` / `Summary` | Next-step honesty |
| `copy.tunnelLocalFirstTitle` / `Summary` | Tunnel present but local broken |
| `path.expectationLocalFirst` | Teaser append when local broken |
| `path.expectationFallbackTor` / `Vpn` / `Proxy` | Teaser if no backend expectation string |

## Acceptance

- [x] Tunnel + local OK → Path teaser shows expectation; next steps Network-only
- [x] No Connect/DNS/benchmark CTA for that honesty case
- [x] Captive / bufferbloat still outrank tunnel in hero
- [x] en + zh-Hans keys present
