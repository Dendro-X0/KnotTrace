# Product plan — Path / exit clarity (proxy·VPN power users)

**Date (UTC):** 2026-11-06  
**Status:** **Complete @ v1.10.2** (P1–P3 · U1–U3 · L1 · D1 · V\*)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Baseline:** **v1.10.1** — desktop trust + Path Truth Quick honesty  
**Depends on:** [product-plan-desktop-trust-path-truth-2026-11.md](./product-plan-desktop-trust-path-truth-2026-11.md), [slow-speed-triage.md](./slow-speed-triage.md), tunnel / reachability / egress designs under `specs/backend/`

```text
GOAL:     Answer “is this this node/path, or my line?” in under ten seconds for
          proxy/VPN-heavy users — without fake Mbps or auto node thrash.
POSTURE:  Observe-first · indicators only · reversible assists · no fake Mbps
DONE:     P1–P3 · U1–U3 · L1 · D1 · V* (v1.10.2)
NEXT:     After CI Release: verify-release-assets.sh v1.10.2
PARKED:   P2b CompanionShell device proof · Obscur · AV · auto node rotation
```

---

## 1. Why this charter

v1.10.1 made Quick trustworthy (ranking, captive/bufferbloat/tunnel honesty). Remaining bounce risk for geo-unlock / proxy users:

| Gap | User pain |
|-----|-----------|
| Site/geo failure not a job | “Only some sites fail” → noise, not Direct vs proxy vs destination |
| Weak exit/session memory | Can’t see “worse after this egress/path” |
| Tunnel ∩ bufferbloat | One shape wins; dual pain understated |
| Connect assumes Clash-like API | Commercial VPN users feel unsupported |
| Advanced evidence overlap | Path break / tunnel / pool / egress compete |
| Locale / first-run | Probes English; tryout still speed-booster-shaped |

Do **not** open mobile L3, AV, or auto rotation under this charter.

---

## 2. Binding rules

| Rule | Meaning |
|------|---------|
| No Obscur coupling | Separate products |
| Indicators only | Absence ≠ unobserved / clean |
| No guaranteed Mbps | Proof and expectation, not acceleration |
| No AV/EDR expansion | Script-risk stays bounded |
| Quick stays default | New depth behind Advanced or one teaser line |
| Specs before diffs | Backend design when classifiers/copy change; frontend spec before JSX |
| Subtraction first | Prefer collapsing panels over new parallel dashboards |

**Deferred forever (this charter):** auto node rotation, continuous packet capture, AV certification, silent OS mutation, Obscur merge, “you are unmonitored,” guaranteed geo-unblock.

---

## 3. Track P — Path / exit clarity

| Order | Band | Outcome |
|------:|------|---------|
| 1 | **P1** Site / geo failure Quick playbook | When reachability is partial: plain Quick verdict (local / DNS / active path / destination-or-policy) + one primary next step; Advanced keeps evidence |
| 2 | **P2** Path / exit journal | Compact recent history: active path, egress IP (if known), slowdown shape, score grade — Overview Quick teaser or Advanced card; no new probe engines required if history fields exist |
| 3 | **P3** Tunnel ∩ bufferbloat Quick line | When both tunnel-active and bufferbloat moderate/severe: one combined hero/next-step line (overhead vs under-load queue vs both) |

---

## 4. Track U — UX clarity

| Order | Band | Outcome |
|------:|------|---------|
| 4 | **U1** Connect unsupported honesty | If no controllable proxy API: calm empty state — diagnosis-only; what *is* measured; no fake Apply |
| 5 | **U2** Path evidence focus | Advanced Network Path section: one “start here” panel recommendation (or quiet accordion default); subtract duplicate CTAs |
| 6 | **U3** Tryout one-liner | COMMUNITY + optional first Overview tip: job = path truth, not speed boost |

---

## 5. Track L — Locale / copy

| Order | Band | Outcome |
|------:|------|---------|
| 7 | **L1** Probe-language honesty | Either localize high-traffic diagnosis summaries used in Quick, or explicitly label Advanced probes “English measurements” in zh-Hans — no silent mixed UI |

---

## 6. Combined ship order

| Order | Band | Track | Status |
|------:|------|-------|--------|
| 1 | **P1** Site/geo Quick playbook | P | Done |
| 2 | **P2** Path/exit journal | P | Done |
| 3 | **P3** Tunnel ∩ bufferbloat | P | Done |
| 4 | **U1** Connect unsupported | U | Done |
| 5 | **U2** Path evidence focus | U | Done |
| 6 | **U3** Tryout one-liner | U | Done |
| 7 | **L1** Probe-language honesty | L | Done |
| 8 | **D1** Docs stamp | Docs | Done |
| 9 | **V\*** Share patch **v1.10.2** | Release | Gate green · tagging |

Interleave: finish **P1** design before UI; **U3** may ship as docs-only anytime after P1 copy exists; **L1** after P1–P3 strings settle.

---

## 7. Success criteria

- [x] On partial site failure, Quick names the likely layer in plain language (not only `partial_site_failure` badge)
- [x] User can see recent path/egress/shape without opening raw history DB
- [x] Tunnel + bufferbloat both evidenced → Quick does not pretend it’s only one
- [x] No-API Connect state never shows a dead Apply primary
- [x] First-run / COMMUNITY one-liner states path-truth job
- [x] No new safe/clean/unmonitored/guaranteed-Mbps copy
- [x] Community gate green before any share tag (**V\***)

---

## 8. Proof layers

| Layer | Proof |
|-------|--------|
| L1 | Unit tests for new ranking/playbook/journal mapping |
| L2 | `bash scripts/community-gate.sh` (or R1) after UI/logic |
| L3 | Manual: Direct OK · proxy partial sites · Tor up+local OK · Tor+bufferbloat · Connect with API down |

---

## 9. Related docs

- [slow-speed-triage.md](./slow-speed-triage.md)
- [product-plan-desktop-trust-path-truth-2026-11.md](./product-plan-desktop-trust-path-truth-2026-11.md)
- [../specs/backend/site-geo-quick-playbook-design.md](../specs/backend/site-geo-quick-playbook-design.md) — P1
- [../specs/backend/path-exit-journal-design.md](../specs/backend/path-exit-journal-design.md) — P2
- [../specs/backend/tunnel-bufferbloat-overlap-design.md](../specs/backend/tunnel-bufferbloat-overlap-design.md) — P3
- [../specs/backend/connect-unsupported-honesty-design.md](../specs/backend/connect-unsupported-honesty-design.md) — U1
- [../specs/backend/path-evidence-focus-design.md](../specs/backend/path-evidence-focus-design.md) — U2
- [../specs/backend/tryout-one-liner-design.md](../specs/backend/tryout-one-liner-design.md) — U3
- [../specs/backend/probe-language-honesty-design.md](../specs/backend/probe-language-honesty-design.md) — L1
- [frontend-spec-next-step-ranking-2026-11.md](./frontend-spec-next-step-ranking-2026-11.md)
- [../specs/backend/tunnel-tor-quick-honesty-design.md](../specs/backend/tunnel-tor-quick-honesty-design.md)
- [../specs/backend/path-break-classifier-design.md](../specs/backend/path-break-classifier-design.md)
- [l3-device-proof.md](./l3-device-proof.md) — P2b remains parked

_Last updated: 2026-11-07T06:30:00Z_
