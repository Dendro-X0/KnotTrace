# Product plan — Desktop trust (A) + Path Truth sharpening (B)

**Date (UTC):** 2026-11-05  
**Status:** **Complete** (A1–A3 · B1–B3 · D1 · V\* prep as **v1.10.1**)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Baseline:** **v1.10.0** — Quick/Advanced, Settings/i18n, Protect calm, bounded script-risk  
**Depends on:** [frontend-spec-ux-pass-pages-2026-11.md](./frontend-spec-ux-pass-pages-2026-11.md), [product-plan-path-truth-2026-10.md](./product-plan-path-truth-2026-10.md), [slow-speed-triage.md](./slow-speed-triage.md)

```text
GOAL:     Make the desktop app feel trustworthy and calm for strangers (A),
          and make Path Truth answers sharper when the path is the problem (B).
POSTURE:  Observe-first · indicators only · reversible assists · no fake Mbps
DONE:     Charter · A1 · A2 · A3 · B1 · B2 · B3 · D1 · V* prep (v1.10.1)
NEXT:     Successor complete @ v1.10.2 — [product-plan-path-exit-clarity-2026-11.md](./product-plan-path-exit-clarity-2026-11.md) · [product-plan-share-patch-1-10-2-2026-11.md](./product-plan-share-patch-1-10-2-2026-11.md)
PARKED:   P2b CompanionShell device proof · Obscur · AV · auto node rotation
```

---

## 1. Why these two tracks

v1.10.0 shipped a **consumer-facing shell** on top of a mature probe/diagnosis core. Remaining risk is not “missing pillars” — it is:

| Track | Risk if ignored | Opportunity |
|-------|-----------------|-------------|
| **A — Desktop trust** | Strangers bounce on dense Advanced pages, weak empty CTAs, or inconsistent Quick jobs | Keep tryouts; feedback becomes actionable |
| **B — Path Truth sharpen** | Users see score + jargon but miss the *right* next step (captive, bufferbloat, tunnel) | Differentiation vs speed apps without lying |

Do **not** start a third parallel charter (mobile L3, CLI productization) until A+B share or explicitly park.

---

## 2. Binding rules

| Rule | Meaning |
|------|---------|
| No Obscur coupling | Separate products |
| Indicators only | Absence ≠ unobserved / clean / virus-free |
| No guaranteed Mbps | Recommendations and proof, not promises |
| No AV/EDR expansion | Script-risk stays bounded indicators |
| Quick stays default | Advanced depth must not leak above the fold |
| Specs before diffs | Frontend notes for A; backend triage/copy for B when ranking changes |

**Deferred forever (this charter):** auto node rotation, continuous packet capture, AV certification, silent OS mutation, Obscur merge, “you are unmonitored.”

---

## 3. Track A — Desktop trust

**Job:** One calm product strangers keep using. Finish UX-pass **P2** and harden Settings/empty states without new probes.

### Outcomes

- Empty / error / “no indicators” states each have **one** clear primary CTA.
- Advanced **Network** is navigable (anchors or section groups), not an avalanche.
- Settings is the single place for theme / language / density / Protect opt-ins; chrome stays quiet in Quick.
- No double heroes or competing “Run check” actions on Overview Quick.

### Ship order (A)

| Order | Band | Outcome |
|------:|------|---------|
| 1 | **A1** Empty & failure CTAs | Spec + implement: integrity empty, check failure, script-risk empty, export-without-report — one primary action each |
| 2 | **A2** Advanced Network navigation | Section anchors or grouped collapsibles (Link / Path / Integrity / Throughput); Quick unchanged |
| 5 | **A3** Settings & chrome consistency | **Done** — Quick chrome quiet (no monitor toggle / no double Run check); next-step i18n en+zh-Hans |

A3 runs **after** first B slice so Path Truth copy lands in locales once.

---

## 4. Track B — Path Truth sharpening

**Job:** When the bottleneck is path/shape (not “buy faster ISP”), Quick Overview names it and offers the right 1–3 next steps.

### Focus slices (not new probe engines)

| Slice | User sees today | Target |
|-------|-----------------|--------|
| **Next-step ranking** | Sometimes generic “run check” / assist noise | Rank by diagnosis confidence + symptom shape ([slow-speed-triage.md](./slow-speed-triage.md)) |
| **Captive / guest / portal** | Detected in context | Plain recovery copy + single next step (complete portal / switch network) — no bypass |
| **Bufferbloat under load** | Exists in probes | Promote in Quick hero / next steps when moderate/severe — never Mbps claim |
| **Tunnel / Tor honesty** | Path-break + tunnel compare exist | Stronger Quick teaser when tunnel present: “slower is expected” vs “local broken” |

### Ship order (B)

| Order | Band | Outcome |
|------:|------|---------|
| 3 | **B1** Next-step ranking pass | **Done** — shared `nextStepPlan` ranks by confidence + shape; Assist eligibility gate; Quick/Advanced same primary |
| 4 | **B2** Captive + bufferbloat Quick surfacing | **Done** — captive portal/Network guidance (no DNS); bufferbloat named in Quick hero/next steps |
| 6 | **B3** Tunnel / Tor Quick honesty | **Done** — Path teaser expectation; tunnel+local OK → Network-only honesty (no Connect speed CTA) |

**Out of scope for B:** new packet capture, router SQM remote config, Tor control, auto DNS/proxy beyond existing Protect gates.

---

## 5. Combined execution order

| Order | Band | Track | Status |
|------:|------|-------|--------|
| 1 | **A1** Empty & failure CTAs | A | **Done** |
| 2 | **A2** Advanced Network navigation | A | **Done** |
| 3 | **B1** Next-step ranking | B | **Done** |
| 4 | **B2** Captive + bufferbloat Quick | B | **Done** |
| 5 | **A3** Settings & chrome / i18n | A | **Done** |
| 6 | **B3** Tunnel / Tor Quick honesty | B | **Done** |
| 7 | **D1** Docs stamp (START-HERE / COMMUNITY / CHANGELOG Unreleased) | Docs | **Done** |
| 8 | **V\*** Optional share patch when A1–B3 feel tryout-ready | Release | **Prep done** (v1.10.1; tag pending) |

Interleave A then B early so trust polish lands before ranking changes; finish A3 after copy settles.

---

## 6. Success criteria

**Track A**

- [x] Each major empty/error surface has exactly one primary CTA in Quick
- [x] Advanced Network has jumpable sections; Quick Network viewport unchanged in job
- [x] Theme / language / density / Protect opt-ins do not duplicate confusingly across Sidebar and Settings

**Track B**

- [x] On captive detection, Quick next steps lead with portal/network guidance (not DNS apply as primary)
- [x] On bufferbloat moderate/severe, Quick names under-load lag (not “slow ISP”)
- [x] When Tor/VPN path is active and local looks OK, Quick does not imply local repair will “speed up” the tunnel

**Shared**

- [x] No new “safe / clean / unmonitored / guaranteed Mbps” copy
- [x] Community gate still green before any share tag (re-run immediately before **V\*** tag)

---

## 7. Proof layers

| Layer | Proof |
|-------|--------|
| L1 | Unit tests for next-step ranking / diagnosis → action mapping where logic moves |
| L2 | `bash scripts/community-gate.sh` (or R1) after UI + ranking changes |
| L3 | Manual Quick scenarios: healthy home · captive · bufferbloat · Tor SOCKS up — screenshot or notes optional |

---

## 8. Related docs

- [frontend-spec-ux-pass-pages-2026-11.md](./frontend-spec-ux-pass-pages-2026-11.md) — P2 polish items this plan absorbs
- [frontend-spec-consumer-calm-protect-2026-11.md](./frontend-spec-consumer-calm-protect-2026-11.md)
- [frontend-spec-ui-density-modes.md](./frontend-spec-ui-density-modes.md)
- [slow-speed-triage.md](./slow-speed-triage.md)
- [../specs/backend/local-first-improvement-roadmap.md](../specs/backend/local-first-improvement-roadmap.md)
- [l3-device-proof.md](./l3-device-proof.md) — P2b remains parked

_Last updated: 2026-11-06T03:20:00Z_
