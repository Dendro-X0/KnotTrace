# Product plan — Share-loop ops (post community readiness)

**Date (UTC):** 2026-11-02  
**Status:** **Complete** (successor to community readiness; P2b unchanged)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Community readiness complete @ **v1.9.1** — [product-plan-community-readiness-2026-11.md](./product-plan-community-readiness-2026-11.md)

```text
GOAL:     Keep the community share loop honest and cheap to operate —
          one pre-share gate, triage helpers for pasted diagnostics,
          docs that match shipped proof (no device hands required).
POSTURE:  Observe-first · claim discipline · reversible assists · no fake Mbps
BASELINE: v1.9.1 + P2a CI headless (Win/macOS/Linux)
DONE:     Charter · S1 · S2 · S3 · S4
NEXT:     Paste/release trust — [product-plan-paste-release-trust-2026-11.md](./product-plan-paste-release-trust-2026-11.md)
```

---

## 1. Why this plan

Community readiness shipped install kits, issue templates, failure UX, and automated L3a. The next risk is **ops friction**, not missing Path Truth features:

| Risk | Why it matters |
|------|----------------|
| Maintainer forgets R1 vs R4 before tagging | Share tags drift from gates |
| Pasted diagnostic JSON hard to triage | Feedback becomes noise |
| Docs drift (roadmap vs matrix) | Honesty burn |
| P2b still parked | Must not imply phone readiness |

**Maintainer constraint (unchanged):** prefer automation and docs; no physical CompanionShell requirement in this charter.

Deferred forever (unchanged): auto node rotation, AV certification, continuous packet capture, Obscur merge.

---

## 2. Tracks

| Band | Outcome |
|------|---------|
| **S1** | `./scripts/community-gate.sh` runs R1 then R4; documented as pre-share default |
| **S2** | `./scripts/validate-diagnostic-bundle.py` (or sh+python) validates a pasted/file bundle for required schema keys |
| **S3** | COMMUNITY / START-HERE / development sync: point at community-gate + P2a CI proof; no phone claims |
| **S4** | Stale-doc pass: local-first roadmap DNS apply table matches [platform-support.md](./platform-support.md) |

---

## 3. Ship order (binding)

| Order | Band | Status |
|------:|------|--------|
| 1 | **S1** Combined community gate | **Done** |
| 2 | **S2** Bundle triage validator | **Done** |
| 3 | **S3** Public docs sync | **Done** |
| 4 | **S4** Stale roadmap honesty | **Done** |

Do not invent product features under this charter. Device L3 remains [l3-device-proof.md](./l3-device-proof.md) P2b.

---

## 4. Success criteria

- [x] One script covers pre-share R1+R4  
- [x] Maintainer can validate a community-pasted diagnostic JSON offline  
- [x] Public docs match matrix + CI proof  
- [x] No new phone-readiness claims  

---

## 5. Doc index

- [product-plan-community-readiness-2026-11.md](./product-plan-community-readiness-2026-11.md) — prior (complete @ v1.9.1)  
- [l3-device-proof.md](./l3-device-proof.md) — P2a/P2b  
- [development.md](./development.md) — gates  
- [COMMUNITY.md](./COMMUNITY.md) — public entry  

_Last updated: 2026-11-02T18:15:00Z_
