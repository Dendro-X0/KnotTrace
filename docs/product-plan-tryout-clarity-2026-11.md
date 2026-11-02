# Product plan — Tryout clarity & release verify

**Date (UTC):** 2026-11-02  
**Status:** **Complete** (successor to share-ops)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Share-ops complete — [product-plan-share-ops-2026-11.md](./product-plan-share-ops-2026-11.md)

```text
GOAL:     Make a stranger’s first session match the docs, and let maintainers
          verify a GitHub Release has the expected desktop installers without
          hand-clicking the Releases page.
POSTURE:  Observe-first · claim discipline · no phone readiness · no fake Mbps
BASELINE: v1.9.1 + community gate + P2a CI headless
DONE:     Charter · F1 · F2 · F3 · F4
NEXT:     Successor — [product-plan-paste-release-trust-2026-11.md](./product-plan-paste-release-trust-2026-11.md)
```

---

## 1. Why this plan

Share-ops made gates and triage cheap. Remaining first-impression risk is **label drift** (docs say “Copy JSON” vs UI) and **release asset doubt** before pointing a community at a tag.

| Risk | Why it matters |
|------|----------------|
| COMMUNITY steps ≠ UI chrome | Tryouts bounce or file “can’t find X” noise |
| Cold Overview copy vague | Users don’t know the next click |
| Asset check is manual | Easy to share a tag missing an OS installer |

**Maintainer constraint:** no physical CompanionShell (P2b still parked). No new Path Truth feature bands.

Deferred forever (unchanged): auto node rotation, AV certification, continuous packet capture, Obscur merge.

---

## 2. Tracks

| Band | Outcome |
|------|---------|
| **F1** | `scripts/verify-release-assets.sh [tag]` uses `gh` to assert Windows MSI+NSIS, Linux AppImage+deb+rpm, macOS dmg, `latest.json` present |
| **F2** | COMMUNITY first-five / export steps use exact UI labels |
| **F3** | Cold Overview hero copy names the primary action (Run health check / Ctrl+K) |
| **F4** | Optional hook: `community-gate.sh` runs F1 when `VERIFY_RELEASE_TAG` is set |

---

## 3. Ship order (binding)

| Order | Band | Status |
|------:|------|--------|
| 1 | **F1** Release asset verifier | **Done** (verified `v1.9.1`) |
| 2 | **F2** COMMUNITY label sync | **Done** |
| 3 | **F3** Cold Overview copy | **Done** |
| 4 | **F4** Gate hook | **Done** |

---

## 4. Success criteria

- [x] `bash scripts/verify-release-assets.sh v1.9.1` exits 0 against current GitHub Release  
- [x] COMMUNITY panel/button names match desktop UI  
- [x] Cold Overview points at Run health check  
- [x] No new phone-readiness claims  

---

## 5. Doc index

- [product-plan-share-ops-2026-11.md](./product-plan-share-ops-2026-11.md) — prior  
- [COMMUNITY.md](./COMMUNITY.md) — public entry  
- [l3-device-proof.md](./l3-device-proof.md) — P2b parked  
- [development.md](./development.md) — F1 + community gate  

_Last updated: 2026-11-02T18:10:00Z_
