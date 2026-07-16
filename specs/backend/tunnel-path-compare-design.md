# Tunnel path compare (L4 design)

Status: **implemented** (band L4). See also [local-first-improvement-roadmap.md](./local-first-improvement-roadmap.md).

## Goal

When Tor, VPN, and/or system proxy are present, show an honest **path compare** so users stop blaming the local NIC for Tor/VPN latency.

## Paths

| Path | When probed |
|------|-------------|
| **Direct** | Always in compare (HTTP client with proxy disabled) |
| **System proxy** | When system HTTP/SOCKS proxy is enabled |
| **Tor SOCKS** | When Tor SOCKS is detected and reachable |

VPN (WireGuard/TUN) often captures all OS traffic, so Direct and System may look similar — copy must say so. Do **not** claim we can accelerate Tor.

## Signals per path

- Sample HTTPS reachability (2–3 domains)
- Median latency when successful
- Optional egress IP (reuse cloudflare trace once per path)
- Tor-only site failures list

## Expectation copy (required)

- Tor: slower latency and lower throughput are expected; do not chase Mbps
- SOCKS down: Tor client / bootstrap issue, not “internet dead”
- VPN: privacy tunnel overhead is normal

## Non-goals

- Controlling Tor Browser / tor daemon
- Auto-switching Tor exits
- Throughput saturation tests on Tor
