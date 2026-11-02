# Product plan — Community readiness & reliability

**Date (UTC):** 2026-11-02  
**Status:** **Complete** for share posture (P2b physical CompanionShell parked)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Expansion complete at **v1.9.0** — [product-plan-cross-platform-capability-2026-10.md](./product-plan-cross-platform-capability-2026-10.md)

```text
GOAL:     Make KnotTrace reliable and honest enough to put in front of an
          online community for voluntary tryouts and feedback — without
          requiring the maintainer to hand-test every surface first.
POSTURE:  Observe-first · claim discipline · reversible assists · no fake Mbps
BASELINE: Path Truth + expansion (CompanionShell, C1–C4, OS matrix) at v1.9.0
DONE:     Charter · R1–R4 · C1–C3 · P1 · C4 (v1.9.1) · P2a CI headless
NEXT:     P2b physical CompanionShell when device capacity returns
```

---

## 1. Why this plan

Capability charters shipped a lot of Path Truth and cross-platform depth. The next risk is not “missing features” — it is **first-impression reliability** when strangers install a release:

| Risk | Why it matters for community |
|------|------------------------------|
| Silent crash / hung check | Users bounce and never return |
| Confusing empty/error states | Feedback becomes “it doesn’t work” with no actionable detail |
| Over-claims in README / release notes | Trust burn; honesty posture must match the product |
| Hard install / no clear “try this” | Community never gets a clean entry path |
| No intake for reports | Useful feedback dies in chat noise |

**Maintainer constraint (binding for this charter):** you may be **unable to run interactive device/UI testing for a stretch**. Bands must prefer **automated gates, CI artifacts, and docs** that raise confidence **without** your hands on every panel. When you regain capacity, optional maintainer L3 is welcome — it is not a blocker for early community share if automated gates are green.

---

## 2. Independence & claim rules (unchanged)

| Rule | Meaning |
|------|---------|
| No Obscur coupling | Separate products |
| Indicators only | Absence ≠ unobserved / secure |
| No guaranteed Mbps | Recommendations, not promises |
| No AV/EDR claims | Startup/process scans stay opt-in indicators |

Deferred forever for this charter (same list): auto node rotation, AV certification, continuous packet capture, Obscur merge.

---

## 3. Tracks

### Track R — Reliability (automation-first)

| Band | Outcome |
|------|---------|
| **R1** | Document and enforce a **cheap reliability gate**: `cargo test -p network-core --lib`, `cargo check -p network-desktop`, `apps/desktop` `tsc`/`npm run build`, existing CI green on `main` |
| **R2** | Harden **failure UX**: health-check fail, export-without-report, updater errors, empty integrity — clear copy, no silent blank screens |
| **R3** | **Crash / panic hygiene**: review unwrap hotspots on startup + check path; prefer Result surfacing to UI; no new silent `unwrap` on those paths |
| **R4** | **Headless regression pack**: scripted `network-probe check --fast --json` (+ optional `--bundle`) as a documented smoke; fail if JSON unparsable or score/integrity missing |

### Track C — Community surface

| Band | Outcome |
|------|---------|
| **C1** | **Public entry kit**: README (or COMMUNITY.md) with what KnotTrace is / is not, Windows install from GitHub Release, first five minutes, how to export a diagnostic bundle for help |
| **C2** | **Feedback intake**: single preferred channel (GitHub Issues template: OS, version, what happened, redacted bundle optional) + short “how to report” in START-HERE |
| **C3** | **Release honesty**: release notes / README claim only matrix-backed OS cells; CompanionShell marked “desktop preview / mobile packaging; device proof pending” until L3 exists |
| **C4** | **Share package**: tag a community-facing release (patch or minor) only after R1 + C1–C3; include link to issues template |

### Track P — Proof without your hands

| Band | Outcome |
|------|---------|
| **P1** | Confirm **v1.9.0** (or successor) GitHub Release has desktop installers; note mobile skip vs publish from Actions logs |
| **P2** | Split: **P2a** macOS/Linux headless probe in CI; **P2b** physical CompanionShell L3 — P2b queued for device capacity |

---

## 4. Ship order (binding)

| Order | Band | Status |
|------:|------|--------|
| 1 | **R1** Automated reliability gates | **Done** |
| 2 | **C1** Public entry kit | **Done** |
| 3 | **C2** Feedback intake (issue templates) | **Done** |
| 4 | **R2** Failure UX hardening | **Done** |
| 5 | **R4** Headless regression pack | **Done** |
| 6 | **R3** Panic hygiene (startup + check path) | **Done** |
| 7 | **C3** Release honesty pass | **Done** |
| 8 | **P1** Release asset verification | **Done** (v1.9.0) |
| 9 | **C4** Community share release | **Done** (v1.9.1) |
| 10 | **P2a** CI headless probe (Linux + macOS) | **Done** |
| 11 | **P2b** Physical CompanionShell L3 | Parked — [l3-device-proof.md](./l3-device-proof.md) |

Do not invent large new feature bands under this charter. Feature ideas from community feedback get a **later** charter.

---

## 5. Proof posture

| Layer | Who runs it | Expectation |
|-------|-------------|-------------|
| L1 | Agent / CI | Unit tests; no new unwrap on named hotspots without justification |
| L2 | CI + agent | `cargo test -p network-core --lib`; desktop build; CI green |
| L3a | Headless | `network-probe check --fast --json` parses; required keys present |
| L3b | Maintainer (optional) | GUI click-smoke / CompanionShell on device — **not required** for C4 |
| L4 | Release | Tagged release with CHANGELOG + installer assets verified (P1) |

**Claim language:** “community-ready” means R1 + C1–C3 + P1 + C4 done. It does **not** mean every OS/device was personally tested by you.

---

## 6. Success criteria (plan-level)

- [x] Stranger can install from GitHub Release and complete a check without reading internal specs  
- [x] Failed checks and export errors show actionable copy  
- [x] Issue template exists and is linked from README / START-HERE  
- [x] Public claims match [platform-support.md](./platform-support.md)  
- [x] Automated gates (R1 + R4) documented; run on/near share tag  

---

## 7. Doc index

- [product-plan-path-truth-2026-10.md](./product-plan-path-truth-2026-10.md) — Path Truth (complete)  
- [product-plan-cross-platform-capability-2026-10.md](./product-plan-cross-platform-capability-2026-10.md) — expansion (complete @ v1.9.0)  
- [platform-support.md](./platform-support.md) — OS matrix  
- [release.md](./release.md) / [github-ci.md](./github-ci.md) — publish path  
- [START-HERE.md](./START-HERE.md) — maintainer + power-user boot  
- [COMMUNITY.md](./COMMUNITY.md) — public tryout kit (C1)  

### P1 verification (2026-11-02)

[v1.9.0](https://github.com/Dendro-X0/KnotTrace/releases/tag/v1.9.0) Release run succeeded with desktop installers published:

| Desktop | Assets present |
|---------|----------------|
| Windows | `KnotTrace_1.9.0_x64_en-US.msi`, `KnotTrace_1.9.0_x64-setup.exe` (+ `.sig`) |
| Linux | AppImage, `.deb`, `.rpm` (+ `.sig`) |
| macOS | `KnotTrace_1.9.0_aarch64.dmg`, `KnotTrace_aarch64.app.tar.gz` (+ `.sig`) |
| Updater | `latest.json` |

Mobile: **Build Android** and **Build iOS** jobs were **skipped** (signing secrets path) — packaging only; device L3 still pending. Matches honesty posture in C3.

### P2a verification

CI job **Headless probe** runs `./scripts/headless-regression.sh` on `ubuntu-latest` and `macos-latest`. Physical CompanionShell remains P2b — see [l3-device-proof.md](./l3-device-proof.md).

_Last updated: 2026-11-02T17:40:00Z_
