# Product plan — Paste safety & release/updater trust

**Date (UTC):** 2026-11-02  
**Status:** **Complete** (successor to tryout clarity; P2b unchanged)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Tryout clarity complete — [product-plan-tryout-clarity-2026-11.md](./product-plan-tryout-clarity-2026-11.md)

```text
GOAL:     Keep community pastes safe to triage and release/updater claims
          honest — without device hands or new Path Truth features.
POSTURE:  Observe-first · claim discipline · no phone readiness · no fake Mbps
BASELINE: v1.9.1 + community gate + F1 asset verify + P2a CI headless
DONE:     Charter · H1 · H2 · H3 · H4
NEXT:     Complete (P2b still parked; multi-OS updater merge deferred)
```

---

## 1. Why this plan

Tryout clarity aligned labels and installer asset names. Remaining share risk is **trust**:

| Risk | Why it matters |
|------|----------------|
| Pasted diagnostic JSON could leak hostname/SSID | Maintainer triage / public issues become liability |
| Asset verify ignores `.sig` / `latest.json` shape | Easy to share a tag with broken updater trust |
| Docs imply multi-OS in-app update | `latest.json` is Windows-job only today — honesty gap |

**Maintainer constraint:** no physical CompanionShell (P2b parked). No new Path Truth feature bands.

Deferred forever (unchanged): auto node rotation, AV certification, continuous packet capture, Obscur merge.

---

## 2. Tracks

| Band | Outcome |
|------|---------|
| **H1** | `validate-diagnostic-bundle.py` rejects hostname keys / default-SSID inclusion; documents fail reasons |
| **H2** | `verify-release-assets.sh` requires installer `.sig` companions + `latest.json` version match + Windows updater platform keys; docs state Windows-primary in-app update |
| **H3** | Fixture bundle under `scripts/fixtures/` exercised by validator; `community-gate.sh` always runs H1 on the fixture (cheap, offline) |
| **H4** | COMMUNITY / development / updater-signing honesty sync; stamp prior plans’ NEXT |

---

## 3. Ship order (binding)

| Order | Band | Status |
|------:|------|--------|
| 1 | **H1** Paste redaction gate | **Done** |
| 2 | **H2** Release sig + latest.json trust | **Done** |
| 3 | **H3** Fixture + community-gate hook | **Done** |
| 4 | **H4** Docs honesty stamp | **Done** |

Do not expand into multi-OS updater merge in this charter (separate later if needed). Device L3 remains [l3-device-proof.md](./l3-device-proof.md) P2b.

---

## 4. Success criteria

- [x] Validator fails a bundle that includes `hostname` or default `include_ssid: true`  
- [x] `verify-release-assets.sh v1.9.1` asserts `.sig` + Windows `latest.json` platforms  
- [x] Community gate runs fixture validation offline  
- [x] Public docs do not claim macOS/Linux in-app update parity  

---

## 5. Doc index

- [product-plan-tryout-clarity-2026-11.md](./product-plan-tryout-clarity-2026-11.md) — prior  
- [COMMUNITY.md](./COMMUNITY.md) — public entry  
- [updater-signing.md](./updater-signing.md) — signing  
- [development.md](./development.md) — gates  

_Last updated: 2026-11-02T18:35:00Z_
