# Integrity / exposure board (Path Truth Band 3)

Status: **implemented** (core + UI + Band 3D ARP/cert snapshots + history deltas)  
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
- Full cert inventory UI / CA attribution (“who issued this root”)  
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
├── gateway_arp: GatewayArpSnapshot | null    # Band 3D
├── cert_store: CertStoreSnapshot | null      # Band 3D
├── not_checked: Vec<string>
└── confidence: high | medium | low
```

```text
IntegrityIndicator
├── kind: proxy_enabled | dns_integrity | captive_portal | tls_intercept_hint
│         | high_risk_listener | path_forced_routing
│         | gateway_arp_churn | cert_store_delta
├── severity: info | warning | critical
├── title: string
├── message: string
└── evidence: string
```

```text
GatewayArpSnapshot
├── gateway_ip: string
├── mac: string | null          # normalized aa:bb:... lowercase
├── source: string
└── observed: bool              # false → probe failed; no fake clean

CertStoreSnapshot
├── root_count: u16             # LocalMachine\Root (Windows) / system roots sample
├── fingerprint: string         # stable djb2 hex of sorted thumbprints + count
├── source: string
└── observed: bool
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
| `gateway_arp_churn` | History: same gateway IP, MAC changed (≥2 observed samples) |
| `cert_store_delta` | History: root fingerprint or count changed vs prior observed sample |

## Band 3D — ARP / cert deltas

### Snapshot (every health check)

| Signal | Windows | Linux | macOS |
|--------|---------|-------|-------|
| Gateway ARP | `Get-NetNeighbor` for `environment.default_gateway` (fallback `arp -a`) | `/proc/net/arp` | `arp -n` |
| Trusted roots | `Cert:\LocalMachine\Root` count + sorted-thumbprint fingerprint | omit / `observed: false` (v1) | omit / `observed: false` (v1) |

Failures → `observed: false`, no fake clean bill. Snapshots always attach when a probe was attempted.

### History enrich (desktop monitor, same window as egress = 48)

```text
enrich_integrity_board(&mut report, &history)
```

Rules:

1. Collect prior `integrity.gateway_arp` / `cert_store` with `observed == true` from history (newest-first, exclude current).
2. **ARP churn:** current MAC present; any prior sample with same `gateway_ip` and different MAC → indicator `gateway_arp_churn` (warning).
3. **Cert delta:** current fingerprint/count vs immediately previous observed cert snapshot differs → `cert_store_delta` (warning). Count-only bump without fingerprint change still fires.
4. When ARP snapshot observed → drop “Gateway / ARP identity churn over time” from `not_checked` (we check churn when history exists; snapshot alone still removes the “never look” honesty item and keep process/covert items).
5. When cert snapshot observed → drop “Trusted root certificate store deltas” from `not_checked`.
6. Recompute `indicator_count`, `level`, `title`, `summary`, `confidence` after pushes.

CLI `network-probe` keeps snapshot-only board (no history); churn/delta indicators appear after desktop enrich.

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
# desktop monitor after history load:
enrich_integrity_board(&mut report, &history);
```

After link/path_break available; listener/ARP/cert scans are sync OS reads (like link_facts).

Update Band 2 bundle later to pull `integrity.indicators` into `integrity_flags` (optional follow-up in same band if cheap).

## UI

- Overview: compact Integrity panel with level badge + disclaimer one-liner  
- Network: fuller list of indicators + high-risk listeners + ARP/cert snapshot lines when observed  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core integrity_board` |
| L2 | `cargo test -p network-core --lib` |
| L3 | `network-probe` JSON has `integrity.gateway_arp` / `cert_store`; GUI Integrity panel still visible |

## Acceptance

- [x] Report always includes disclaimer  
- [x] Zero indicators ≠ “safe” copy  
- [x] Proxy / DNS / captive map to indicators  
- [x] Listener scan unit-tested with fixture parsers  
- [x] Docs + product plan Band 3 status  
- [x] L3 (runtime): live `network-probe` includes `integrity` with disclaimer (`unobserved` copy present)  
- [x] L3 (GUI): Integrity panel + disclaimer visible via CDP click-smoke (2026-10-30)  
- [x] Band 3D: ARP + cert snapshots on board; history enrich indicators; unit fixtures for churn/delta  



