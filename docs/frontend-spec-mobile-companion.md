# Frontend spec — Mobile companion shell (Band B1)

Status: **B2 shell landed** (CompanionHome + `get_app_capabilities`; desktop `?companion=1` preview)  
Backend: [specs/backend/mobile-companion-design.md](../specs/backend/mobile-companion-design.md)  
Product plan: [product-plan-cross-platform-capability-2026-10.md](./product-plan-cross-platform-capability-2026-10.md)

## Product job

One screen that answers: **“Is my path broken, and what should I paste into help/AI?”**

## Shell

| Element | Spec |
|---------|------|
| Nav | **No desktop sidebar.** Top bar: brand `KnotTrace`, optional overflow (About / redaction note) |
| Primary page | `CompanionHome` only for v1 |
| Secondary | Modal/sheet for Export preview + About; no Protect/Connect/DNS pages |

## CompanionHome layout (one composition)

Order top→bottom:

1. **Brand + grade** — score number, grade chip, one-line score summary  
2. **Primary CTA** — `Run check` (maps to manual Fast check); shows busy state while running  
3. **Path break** — reuse `PathBreakPanel` compact mode (or strip): active path, break hop, title, confidence  
4. **Integrity** — reuse integrity summary: level, indicator count, disclaimer (always), first 3 indicators max with “more” expand  
5. **Export** — `Copy diagnostic JSON` / `Share` (Android share intent when available)  
6. **Optional** — Local explain collapse (same `explain_locally_report`); off by default collapsed  

**Do not** put stats strips, Connect, Protect toggles, throughput, or benchmark on Home.

## States

| State | UI |
|-------|-----|
| Cold start | Empty score placeholder + CTA; copy “Run a check to measure this path.” |
| Running | CTA disabled / spinner; keep prior panels if present |
| Ready | Score + path + integrity + export enabled |
| Failed | Inline error string from `health-check-failed`; CTA retry |
| Export without report | Disable export; hint to run check first |

## Copy rules (honesty)

- Never “secure / clean / unmonitored.”  
- Integrity empty state: **“No indicators in this check”** + disclaimer.  
- Export footer note: redaction defaults (no hostname; SSID off).

## Desktop coexistence

| Platform | Shell |
|----------|--------|
| Desktop | Existing `App` + `Sidebar` unchanged |
| Mobile | `CompanionShell` when `capabilities.companion_shell === true` |

Shared components: Path break / Integrity panels should accept a `density: "full" | "compact"` prop if needed rather than forking.

## Accessibility

- CTA ≥ 44px touch target  
- Score and errors not color-only (text labels)  
- Export success: brief confirmation toast/text  

## Out of scope visuals

Dark-mode-only redesign, purple glow marketing, multi-page IA, floating promo chips.

## B2 acceptance (UI)

- [x] Cold → Run check → Ready shows grade + path-break title + integrity block (CompanionHome)  
- [x] Export copies JSON with `schema_version` knottrace diagnostic bundle (share sheet deferred)  
- [x] No navigation entry to Protect apply / Connect switch / DNS apply  
- [x] Desktop build still shows full sidebar (`companion_shell` false unless `?companion=1`)  
- [ ] L3 on physical/emulator Android APK
