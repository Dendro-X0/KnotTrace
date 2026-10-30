# Product plan — Cross-platform versatility & capability expansion

**Date (UTC):** 2026-10-30  
**Status:** Chartered (successor to Path Truth maintain track)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Path Truth complete at **v1.8.1** — [product-plan-path-truth-2026-10.md](./product-plan-path-truth-2026-10.md)

```text
GOAL:     Make KnotTrace useful on every desktop OS we ship, plus a thin
          mobile companion, with deeper integrity / timeline / assist coverage
          — without fake Mbps, AV claims, or Obscur coupling.
POSTURE:  Observe-first · indicators with confidence · reversible assists
BASELINE: Path Truth Bands 1–5 + 3D/3E + CLI + AI explain (4A–4C) shipped
DONE:     Charter · A1 · D1/D2 · A3 · C1 hosts/proxy timeline
NEXT:     B1 Mobile companion design
```

---

## 1. Why this plan

Path Truth shipped the honesty ladder (path break, integrity, AI explain, egress timeline, CLI). Remaining gaps that limit **versatility** and **felt utility**:

| Gap class | Examples |
|-----------|----------|
| **OS unevenness** | Process↔egress Windows-only; local caps Windows-only; mobile is packaging without Path Truth UX |
| **Onboarding / discoverability** | START-HERE under-describes Integrity / Explain / Egress / process-egress |
| **Integrity depth** | Wi‑Fi retries / WAN-facing listener hints thin; autoruns still deferred (C4) |
| **Mobile** | APK/IPA pipeline exists; no “check + export bundle” companion surface |

This plan **expands** KnotTrace. It does not reopen Obscur merge, speed optimizers, or AV certification ([Path Truth §9](./product-plan-path-truth-2026-10.md)).

---

## 2. Independence (binding — unchanged)

| Rule | Meaning |
|------|---------|
| No Obscur dependency either way | Separate products |
| No merge charter | No UI bridge / shared monorepo gate |
| Claim discipline | Indicators only; absence ≠ unobserved |

---

## 3. Capability tracks

### Track A — Desktop OS parity (Windows · macOS · Linux)

Goal: every Path Truth and local-assist surface has an honest matrix cell: **Yes / Partial / N/A / Not yet**, then close **Not yet** where feasible.

| Band | Outcome |
|------|---------|
| **A1** | Publish living matrix in [platform-support.md](./platform-support.md) for Path Truth + assists; implement process↔egress on Linux/macOS (best-effort `ss` / `lsof`) |
| **A2** | Local caps **observe** (read-only) on macOS/Linux where APIs exist; repair stays Windows or explicit N/A |
| **A3** | Protect alert mapping for integrity kinds (forced routing, cert delta, ARP churn, process-egress) |

### Track B — Mobile companion (thin wrap)

Not a second full desktop. After A1 matrix is public:

| Band | Outcome |
|------|---------|
| **B1** | Design: Android/iOS shell that runs **fast check**, shows score + path-break + integrity summary, **exports diagnostic bundle** |
| **B2** | Implement Android-first (Tauri mobile); reuse `network-core` |
| **B3** | iOS parity for B1 surfaces; signing already documented |

**Mobile non-goals (v1):** Protect auto-apply, MTU/local-caps repair, Connect node switch, CDP smoke.

### Track C — Capability depth (desktop-first)

| Band | Outcome |
|------|---------|
| **C1** | Config timeline: detect **hosts file** and **system proxy** changes (hash/mtime snapshots), merge into bundle timeline |
| **C2** | Listener board: WAN-facing / non-loopback high-risk emphasis + clearer copy |
| **C3** | Wi‑Fi quality extras where OS allows (retries/noise — indicators only) |
| **C4** | Optional: autoruns / startup network agents (**opt-in**, indicators only) |

### Track D — Docs & discoverability

| Band | Outcome |
|------|---------|
| **D1** | Refresh START-HERE for Path Truth panels + data files (`process_egress_settings.json`, AI explain settings) |
| **D2** | Mark [future-plan-security-integrity-2026-10.md](./future-plan-security-integrity-2026-10.md) as **superseded / absorbed** into Path Truth + this plan |

---

## 4. Ship order (binding)

Execute in order; do not skip to mobile full UI before A1.

| Order | Band | Status |
|------:|------|--------|
| 1 | **A1** Desktop Path Truth parity matrix + process↔egress Linux/macOS | **Done** |
| 2 | **D1** START-HERE refresh | **Done** |
| 3 | **D2** Future-plan status stamp | **Done** |
| 4 | **A3** Protect ↔ integrity alert mapping | **Done** |
| 5 | **C1** Hosts/proxy change timeline | **Done** |
| 6 | **B1** Mobile companion design | **Next** |
| 7 | **B2** Android companion implement | After B1 |
| 8 | **A2 / C2 / C3 / B3 / C4** | As capacity allows |

**Defer (unchanged):** auto node rotation, guaranteed Mbps, AV/EDR, Obscur integration, continuous packet capture.

---

## 5. Proof posture

| Layer | Expectation |
|-------|-------------|
| L1 | Unit tests per band |
| L2 | `cargo test -p network-core --lib` + desktop `tsc` |
| L3 | `network-probe check --json` on each desktop OS we claim; Android smoke for B2 |
| L4 | Release tag only after matrix + CHANGELOG updated |

---

## 6. Success criteria (plan-level)

- [ ] Path Truth features documented per OS with no silent “Windows-only” surprises  
- [ ] Process↔egress works or honestly `not_implemented` with user-visible copy on each desktop OS  
- [ ] Mobile user can run a check and export a redacted bundle without desktop  
- [x] At least one new integrity/timeline signal beyond Path Truth v1.8.1 (C1 or A3)  
- [x] START-HERE matches the product users actually see  

---

## 7. Doc index

- [product-plan-path-truth-2026-10.md](./product-plan-path-truth-2026-10.md) — prior charter (complete)  
- [platform-support.md](./platform-support.md) — living OS matrix (A1 updates)  
- [mobile-signing.md](./mobile-signing.md) — APK/IPA signing  
- [future-plan-security-integrity-2026-10.md](./future-plan-security-integrity-2026-10.md) — absorbed sketch  

_Last updated: 2026-10-30T13:00:00Z_
