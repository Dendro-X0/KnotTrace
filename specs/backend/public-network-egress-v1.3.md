# Public network & egress (v1.3)

## Problem

Hotel, restaurant, and café Wi-Fi often share a private gateway (`192.168.x.x`) so KnotTrace
mislabels them as `home_lan`. Users need:

- Public egress IP visibility and stability checks
- Guest/public network risk classification
- Captive portal detection
- Actionable guidance without forced network or proxy changes

## Principles

1. **Observe first** — classify before acting.
2. **Recommend by default** — proxy node switching is opt-in, not automatic.
3. **Safe auto only** — trusted DNS apply (reversible) remains the primary automation.
4. **Cross-platform** — Win/macOS/Linux share `network-core` probes; DNS apply is OS-specific (`dns_platform`).
5. **No bypass** — do not defeat captive portals, corporate policy, or geo-blocks.

## v1.3.0 scope

| Feature | Behavior |
|---------|----------|
| **Egress probe** | Query 2–3 IP endpoints over the system path; optional Tor SOCKS path when reachable |
| **Network context** | Classify guest Wi-Fi, cellular, home LAN, captive portal |
| **Captive portal hint** | HTTP probes to `generate_204` / Microsoft connect test |
| **Recommendations** | Human-readable guidance panel (no forced WiFi/VPN/proxy changes) |
| **Protect defaults** | `auto_apply_connect` and `auto_recover_site_access` default **off** |
| **Trust model** | Guest Wi-Fi / captive portal → untrusted unless VPN active |

## Data model

- `EgressReport` — primary IPv4, confidence, per-endpoint results, optional Tor path
- `NetworkContextReport` — `kind`, `risk_level`, captive portal status, signal list
- `NetworkRecommendations` — prioritized titles/messages for UI and Protect alerts

## Non-goals (v1.3)

- SSID/BSSID collection (privacy + platform APIs)
- Auto VPN/Tor client control
- Linux/macOS DNS apply — shipped in L1 (`dns_platform` macOS/Linux backends); see [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md)
- Client isolation / ARP probing

## Exit criteria

- Health check includes egress, network context, and recommendations
- Guest Wi-Fi on private gateway classified as untrusted
- `cargo test -p network-core` passes
- Network page shows egress IP and public-network guidance
