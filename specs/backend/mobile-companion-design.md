# Mobile companion (Track B / Band B1 design)

Status: **designed** (B1) — implement in **B2** (Android) / **B3** (iOS)  
Product plan: [product-plan-cross-platform-capability-2026-10.md](../../docs/product-plan-cross-platform-capability-2026-10.md) Track B  
UI blueprint: [frontend-spec-mobile-companion.md](../../docs/frontend-spec-mobile-companion.md)  
Signing: [mobile-signing.md](../../docs/mobile-signing.md)

## Goal

Ship a **thin** Android/iOS companion (same Tauri v2 app family as desktop) that can:

1. Run a **fast** health check (`CheckProfile::Fast`)  
2. Show **score**, **path-break** summary, and **integrity** summary  
3. **Export** a redacted diagnostic bundle (copy / share JSON)

Not a second full desktop product.

## Non-goals (v1 companion)

| Out | Why |
|-----|-----|
| Protect auto-apply / DNS Assist apply | Privilege + UX risk on phone |
| MTU / local-caps repair | Desktop-oriented OS APIs |
| Connect Assist node switch | Clash API usually on desktop localhost |
| Background monitor tray / CDP smoke | Desktop tooling |
| Process↔egress opt-in board | High friction / permission story on mobile |
| Cloud AI explain | Optional later; local explain OK if cheap |

## Architecture

```text
apps/desktop (Tauri + React)
├── cfg(desktop)  → full sidebar (Overview, Network, DNS, Connect, Protect)
└── cfg(mobile)   → CompanionShell: single Home + Export + About
         │
         ▼
network-core::run_health_check_with_settings(..., CheckProfile::Fast)
         │
         ▼
IPC reuse: perform_check / export_diagnostic_bundle / explain_locally_report (optional)
```

**Reuse:** one Rust crate (`network-core`), one frontend package. Mobile is a **navigation + surface subset**, not a fork.

**Detection:** use Tauri mobile platform (`os` / `#[cfg(mobile)]` in Rust; frontend `navigator` or injected `platform: mobile` from init). Prefer a single `isCompanionShell` flag from Rust `get_app_capabilities` rather than duplicating feature detection in many components.

## Canonical owner

| Piece | Owner |
|-------|--------|
| Fast check orchestration | existing `monitor::perform_check` (manual → Fast already) |
| Bundle export | existing `export_diagnostic_bundle` |
| Integrity / path-break | `network-core` (unchanged contracts) |
| Companion nav / pages | new `CompanionHome` (+ hide desktop-only routes) |
| Capabilities flag | `apps/desktop/src-tauri` command or init payload |

## IPC allowlist (B2)

| Command | Mobile |
|---------|--------|
| Manual health check | **Yes** (Fast) |
| `export_diagnostic_bundle` | **Yes** |
| `explain_locally_report` | **Yes** (optional panel) |
| Protect settings / auto-apply | **No** (hide UI; commands may remain but unused) |
| DNS/MTU/local-caps apply | **No** |
| Connect switch / Clash discover | **No** |
| Process-egress settings | **No** |
| Throughput / benchmark | **Defer** (battery; optional later) |

## Probe posture on mobile

| Signal | Expectation |
|--------|-------------|
| Score + probe + diagnosis | Same Fast path as desktop manual |
| Path break | Yes when probe inputs exist |
| Integrity soft signals | Proxy / DNS integrity / captive / TLS hint / forced routing |
| Listener scan | Best-effort; may land in `not_checked` on restricted sandboxes |
| Gateway ARP / cert store | Best-effort; honest `observed: false` when blocked |
| Hosts/proxy watch (C1) | Persist under mobile app data dir when writable |
| SSID in export | Default **off** (same redaction) |

## Data dir

Use Tauri app data directory on device (same `data_dir()` helper). Config-watch / DNS integrity settings files may exist; assists that mutate OS stay unavailable in UI.

## Proof plan (for B2, not B1)

| Layer | Check |
|-------|--------|
| L1 | Unit: companion capability flag defaults; Fast profile still used for manual |
| L2 | `cargo check` android target when SDK present; frontend typecheck |
| L3 | Device: run check → see score/path/integrity → export JSON shares/copies |
| L4 | Signed APK from release job (existing mobile-signing secrets) |

## Claim language

- **B1 designed:** this doc + frontend spec committed; plan stamp Next→B2  
- **B2 implemented:** CompanionShell ships on Android with allowlisted IPC  
- **Verified:** L3 on a physical or emulator device with evidence path in handoff/changelog  

## Open decisions (resolve in B2 if blocking)

1. Share sheet vs clipboard-only for bundle JSON on Android.  
2. Whether background poll runs at all on mobile (default **off** to save battery).  
3. Whether Network page deep-links exist or Home embeds the three panels only.
