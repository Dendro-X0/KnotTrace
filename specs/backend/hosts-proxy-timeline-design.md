# Hosts / proxy change timeline (Band C1)

Status: **implemented** (core + monitor refresh + timeline merge + Protect map)  
Product plan: [product-plan-cross-platform-capability-2026-10.md](../../docs/product-plan-cross-platform-capability-2026-10.md) Track C / Band C1  
Depends on: diagnostic bundle config timeline, integrity board indicators, Protect A3 mapping

## Goal

Detect **hosts file** and **system proxy** changes via hash/mtime (and proxy enabled/server fingerprint) snapshots, persist change events under the app data dir, and merge them into the diagnostic **config timeline**. When a change is first observed, also push integrity indicators (indicator-only language).

## Non-goals

- Blocking edits to hosts or proxy  
- Continuous file watchers / kernel filters  
- Full hosts content export in the diagnostic bundle  
- Guaranteeing detection of every edit if the app never runs a check

## Canonical owner

| Piece | Owner |
|-------|--------|
| Snapshot + persist + events | `network-core::config_watch` |
| Timeline merge | `diagnostic_bundle::build_config_timeline` |
| Indicator kinds | `IntegrityIndicatorKind::{HostsFileDelta, ProxyConfigDelta}` |
| Desktop refresh | `monitor::enrich_history_claims` (after integrity enrich) |
| Protect alerts | existing A3 mapper (warning for both kinds) |

## Data model

```text
HostsFileSnapshot
├── path: string
├── mtime_unix: i64 | null
├── size: u64
├── content_hash: string     # djb2 hex of file bytes (or empty if unread)
└── observed: bool

ProxyWatchSnapshot
├── enabled: bool
├── server_fingerprint: string  # djb2 of enabled|source|server
└── source: string

ConfigWatchState { hosts, proxy, updated_at }

ConfigWatchEvent { timestamp, kind: "hosts"|"proxy", detail: string }
```

Persistence:

- `config_watch_state.json` — last snapshots  
- `config_watch_events.jsonl` — append-only change log (cap ~200)

## Hosts paths

| OS | Path |
|----|------|
| Windows | `%SystemRoot%\System32\drivers\etc\hosts` |
| Linux / macOS | `/etc/hosts` |

Unreadable → `observed: false`; no fake clean; no indicator from unread→unread.

## Change rules

| Signal | Fires when |
|--------|------------|
| Hosts | Prior observed + current observed and (`content_hash` or `mtime_unix`/`size`) differ |
| Proxy | Prior fingerprint ≠ current fingerprint (includes enable/disable and server change) |

First observation (no prior state) seeds state **without** an event.

## Proof

| Layer | Command / check |
|-------|-----------------|
| L1 | Unit: hash change → event; first seed → no event; timeline includes event line |
| L2 | `cargo test -p network-core config_watch` + `diagnostic_bundle` timeline test |
| L3 | Manual: flip system proxy or edit hosts, run check, export bundle sees timeline line |

## Claim language

- **Implemented:** module + timeline merge + indicators + Protect map + tests green  
- **Verified:** L1/L2 named above  
- **Fixed:** N/A (feature)
