# Startup / autorun network agents (Band C4)

Status: **implemented** (opt-in settings + Windows/Linux/macOS scans + Protect toggle)  
Product plan: [product-plan-cross-platform-capability-2026-10.md](../../docs/product-plan-cross-platform-capability-2026-10.md) Track C / Band C4  
Related: [process-egress-design.md](./process-egress-design.md) (same opt-in posture)

## Goal

Opt-in scan for **startup / autorun entries whose names look network-related** (VPN/proxy/tunnel clients). Surface as integrity **indicators only** — never “malware found,” never disable entries.

## Non-goals

- Full autoruns inventory / Sysinternals parity  
- Killing or disabling startup items  
- Continuous watchers  
- AV / reputation lookups  

## Canonical owner

| Piece | Owner |
|-------|--------|
| Scan + settings | `network-core::autoruns` |
| Enrich | desktop `monitor::enrich_history_claims` after process-egress |
| UI toggle | Protect page (alongside process↔egress) |
| Indicator kind | `IntegrityIndicatorKind::StartupNetworkAgent` |

## Settings

`autoruns_settings.json`: `{ enabled, consent_ack }` — both required to scan.  
Consent string: `knottrace.autoruns.consent.v1`.

## Snapshot

```text
AutorunSnapshot { scanned, enabled, source, hits: [{ name, location, command_hint? }], summary }
```

Hits capped at 12. Name/command matched against a small keyword list (vpn, clash, wireguard, tor, …).

## Platform sources (v1)

| OS | Source |
|----|--------|
| Windows | `HKCU\…\Run` value names + commands |
| Linux | `~/.config/autostart/*.desktop` Name/Exec |
| macOS | `~/Library/LaunchAgents/*.plist` basenames (+ Label best-effort) |

## Proof

| Layer | Check |
|-------|--------|
| L1 | Keyword match unit tests; settings require consent |
| L2 | `cargo test -p network-core autoruns` |
| L3 | Enable on Protect → check → integrity shows hit when a matching Run entry exists |
