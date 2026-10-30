# Integrity / exposure board (Path Truth Band 3)

Status: **implemented** (core + UI; ARP/cert deltas still not_checked; L3 smoke pending)  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §6, §8 order 3  
Depends on: Band 1 path-break, Band 2 bundle (will consume integrity flags later)

## Goal

Surface **indicators** of exposure, forced routing, or interception — never a green “safe/unobserved” badge.

Canonical copy (always present on the report):

> We found **indicators** consistent with exposure, forced routing, or interception. **Absence of indicators does not mean you are unobserved.**

When no indicators fire, title/summary still use **“No indicators in this check”** language — not “clean.”

## Non-goals

- AV / malware removal  
- Covert tap detection guarantees  
- Process ↔ egress correlation (opt-in later)  
- Cert-store delta persistence (v1 lists as `not_checked`)  
- ARP churn history (v1 may snapshot gateway MAC once; churn needs store — defer)  
- Cloud AI (Band 4)

## Canonical owner

| Piece | Owner |
|-------|--------|
| Classification + listener scan | `network-core::integrity_board` |
| Orchestration | `run_health_check` after path_break inputs exist |
| UI | Overview/Network **Integrity** panel |

## Data model

```text
IntegrityBoardReport
├── indicator_count: u8
├── level: none | info | caution | elevated   # never "safe"
├── title: string
├── summary: string
├── disclaimer: string                        # canonical copy always
├── indicators: Vec<IntegrityIndicator>
├── listeners: ListenerScanSummary | null
├── not_checked: Vec<string>
└── confidence: high | medium | low
```

```text
IntegrityIndicator
├── kind: proxy_enabled | dns_integrity | captive_portal | tls_intercept_hint
│         | high_risk_listener | path_forced_routing
├── severity: info | warning | critical
├── title: string
├── message: string
└── evidence: string
```

```text
ListenerScanSummary
├── scanned: bool
├── source: string
├── high_risk: Vec<{ port, address, process_hint? }>
├── total_listened_sampled: u8
└── summary: string
```

## v1 signal sources

| Indicator | Source |
|-----------|--------|
| `proxy_enabled` | `environment.proxy.enabled` |
| `dns_integrity` | `dns_integrity` Caution/Suspicious |
| `captive_portal` | `network_context.captive_portal` |
| `tls_intercept_hint` | Dominant TLS failures on reachability/tunnel when proxy/VPN present |
| `high_risk_listener` | Local listen scan (below) |
| `path_forced_routing` | Soft: proxy on + path_break active_path SystemProxy/Vpn |

## Listener scan (lightweight)

High-risk ports (non-loopback binds): `21, 23, 135, 139, 445, 3389, 5900, 5985, 5986, 5600`.

| OS | Method |
|----|--------|
| Windows | `Get-NetTCPConnection -State Listen` via hidden PowerShell |
| Linux | parse `/proc/net/tcp` (+ tcp6 if present) |
| macOS | `netstat -an -p tcp` listen lines |

Failures → `scanned: false`, no fake clean bill. Cap results (e.g. 12 rows).

## Orchestration

```text
report.integrity = Some(assess_integrity_board(&report));
```

After link/path_break available; listener scan is sync OS read (like link_facts).

Update Band 2 bundle later to pull `integrity.indicators` into `integrity_flags` (optional follow-up in same band if cheap).

## UI

- Overview: compact Integrity panel with level badge + disclaimer one-liner  
- Network: fuller list of indicators + high-risk listeners  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core integrity_board` |
| L2 | `cargo test -p network-core --lib` |
| L3 | Dev app shows Integrity panel; disclaimer visible with zero indicators |

## Acceptance

- [x] Report always includes disclaimer  
- [x] Zero indicators ≠ “safe” copy  
- [x] Proxy / DNS / captive map to indicators  
- [x] Listener scan unit-tested with fixture parsers  
- [x] Docs + product plan Band 3 status  
- [x] L3 (runtime): live `network-probe` includes `integrity` with disclaimer (`unobserved` copy present)  
- [x] L3 (GUI): Integrity panel + disclaimer visible via CDP click-smoke (2026-10-30)  



