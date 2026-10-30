# Path-break classifier (Path Truth Band 1)

Status: **implemented** (core + minimal UI; L3 manual pending)  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §8 order 1  
Related: [tunnel-path-compare-design.md](./tunnel-path-compare-design.md), [public-network-egress-v1.3.md](./public-network-egress-v1.3.md)

## Goal

Answer four questions from one structured report on every health check:

1. **Local OK?** — link / captive / DNS / local probe health before blaming tunnels  
2. **Which path?** — Direct / SystemProxy / VpnTunnel / TorSocks (primary active path)  
3. **Where break?** — hop label: Dns / Tcp / Tls / Http / None / Local / Unknown  
4. **Environment class** — home / guest / corp-ish / hotspot / captive / filtered / unknown  

User-facing outcome: plain summary + evidence; never “safe/unobserved”; never guaranteed Mbps.

## Non-goals (this band)

- New heavy probes (reuse existing `HealthReport` signals)  
- Integrity / exposure board (Band 3)  
- AI bundle export (Band 2)  
- Auto node rotation or Tor control  
- Obscur coupling  

## Canonical owner

| Piece | Owner |
|-------|--------|
| Classification | `network-core` module `path_break` |
| Orchestration | `run_health_check_with_settings` after diagnosis inputs exist |
| UI surface | Overview diagnosis / Network path panel (thin consume) |

No parallel writers. Diagnosis may *read* `path_break`; it must not re-derive hop labels ad hoc.

## Data model

`PathBreakReport` on `HealthReport` (optional for serde back-compat):

```text
PathBreakReport
├── local_ok: bool
├── local_issues: Vec<string>          # short evidence
├── active_path: PathBreakPath         # direct | system_proxy | vpn_tunnel | tor_socks
├── paths_present: Vec<PathBreakPath>
├── break_hop: PathBreakHop            # none | local | dns | tcp | tls | http | unknown
├── environment_class: PathEnvironmentClass
├── confidence: high | medium | low
├── title: string
├── summary: string
├── evidence: Vec<string>
└── not_checked: Vec<string>           # honesty list for later AI bundle
```

### Enums

| Enum | Values |
|------|--------|
| `PathBreakPath` | `Direct`, `SystemProxy`, `VpnTunnel`, `TorSocks` |
| `PathBreakHop` | `None`, `Local`, `Dns`, `Tcp`, `Tls`, `Http`, `Unknown` |
| `PathEnvironmentClass` | `HomeLan`, `GuestWifi`, `PublicHotspot`, `CaptivePortal`, `CorporateProxy`, `VpnOnly`, `Tor`, `FilteredRegion`, `Unknown` |

## Input signals (reuse only)

| Signal | Source field |
|--------|----------------|
| VPN / proxy / Tor tags | `environment.tags`, `environment.proxy`, `environment.tor` |
| Gateway / DNS probes | `probe.gateway`, `probe.dns`, `probe.internet` |
| Captive / context | `network_context` |
| Site errors | `site_reachability.results[].error_kind` |
| Proxy vs direct | `proxy_path_report` |
| Tunnel samples | `tunnel_compare` |
| DNS integrity | `dns_integrity` (mismatch → hop bias Dns when local otherwise OK) |
| Link down-ish | `link_facts.issues`, no default route |

## Classification rules (priority order)

### A. Local gate

`local_ok = false` when any of:

- No active/default interface or gateway probe hard-fail  
- Captive confirmed/suspected  
- Majority local DNS probes fail  
- Link facts severity blocking (if already modeled)

If `local_ok = false` → `break_hop = Local` (or `Dns` if only DNS fails), `confidence` medium+, stop path blame.

### B. Active path

1. Tor detected + SOCKS reachable → prefer `TorSocks` when Tor path is the intended privacy path **or** when tunnel_compare Tor is the only failing/succeeding contrast needed — default **present** paths list includes Tor; **active** = Tor if Tor tag and user path is Tor-primary *or* if system direct fails but Tor works is rare; simpler v1:  
   - If Tor detected → include `TorSocks` in `paths_present`  
   - `active_path` = `TorSocks` if Tor SOCKS reachable and (no system proxy OR tor is tagged); else  
   - `VpnTunnel` if `EnvironmentTag::Vpn`  
   - `SystemProxy` if proxy enabled  
   - else `Direct`
2. Always populate `paths_present` from tags/proxy/tor.

### C. Break hop (when local_ok)

Map dominant `SiteReachErrorKind` / tunnel failure notes:

| Dominant evidence | `break_hop` |
|-------------------|-------------|
| Dns / integrity mismatch + reach failures | `Dns` |
| Timeout / connection reset without TLS wording | `Tcp` |
| Tls | `Tls` |
| HttpServer / HttpBlocked | `Http` |
| Proxy error kinds on proxy path only | `Tcp` or `Http` + active_path SystemProxy |
| Tor SOCKS unreachable | `Local` (client/bootstrap) with path TorSocks |
| All success | `None` |
| Mixed / sparse | `Unknown` |

Prefer **proxy_path_report** / **tunnel_compare** when present over single-path site_reachability.

### D. Environment class

| Condition | Class |
|-----------|--------|
| Captive confirmed/suspected | `CaptivePortal` |
| Tor active | `Tor` |
| VPN tag, no captive | `VpnOnly` |
| Proxy enabled + public/guest signals | `CorporateProxy` (soft; may be home proxy) |
| `GuestWifi` / public context | `GuestWifi` / `PublicHotspot` |
| Home LAN context | `HomeLan` |
| Else | `Unknown` |

Do **not** claim `FilteredRegion` unless TLS resets cluster with regional copy already present — v1 may emit only when tunnel_compare + TLS failures dominate; otherwise leave Unknown.

## Copy invariants

- Title/summary name path + hop: e.g. “VPN path fails at TLS”  
- Include at least one evidence string citing measured field  
- `not_checked` always lists deferred items (listeners, cert store, ARP churn) so Band 2/3 can reuse honesty  

Canonical negative: never imply surveillance absence.

## Orchestration

After `tunnel_compare`, `proxy_path_report`, `network_context`, `dns_integrity`, `site_reachability`, `link_facts` are on the report (before or after `diagnose_network`):

```text
report.path_break = Some(classify_path_break(&report));
```

`diagnose_network` / recommendations may consume `path_break` for a single next-step hint — prefer reading the classifier over duplicating hop logic.

## UI (minimal Band 1)

- Overview: show `path_break.title` + summary near diagnosis (or fold into diagnosis panel)  
- Network: small **Path break** strip above Tunnel compare when `break_hop != None` or non-Direct active path  
- Types: mirror structs in `apps/desktop/src/types.ts`

## Proof plan

| Layer | Command / check |
|-------|-----------------|
| L1 | `cargo test -p network-core path_break` — fixture matrix (local fail, proxy TLS, Tor SOCKS down, healthy direct) |
| L2 | `cargo test -p network-core` |
| L3 | Manual: `npm run tauri dev` → Overview shows path_break when VPN/proxy/Tor present |
| L4 | n/a this band |

## Acceptance

- [x] `PathBreakReport` always present on successful health check  
- [x] Local failure never attributed to Tor/VPN hop (unit: `local_dns_failure_not_blamed_on_vpn`)  
- [x] Tor SOCKS down → hop Local + Tor path, expectation-safe copy  
- [x] Unit tests cover ≥4 fixtures (`cargo test -p network-core path_break`)  
- [x] Docs: architecture + START-HERE index mention Path Truth Band 1  
- [ ] L3: `npm run tauri dev` Overview/Network show Path break panel  

