# Product plan — Multi-OS updater manifest

**Date (UTC):** 2026-11-02  
**Status:** **Complete** (successor to paste/release trust; P2b unchanged)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Paste/release trust complete — [product-plan-paste-release-trust-2026-11.md](./product-plan-paste-release-trust-2026-11.md)

```text
GOAL:     Ship one race-free latest.json that covers Windows + Linux + macOS
          so in-app update is not Windows-only — without device hands.
POSTURE:  Observe-first · claim discipline · no phone readiness · no fake Mbps
BASELINE: v1.9.1 + Windows-primary latest.json honesty (H2)
DONE:     Charter · U1 · U2 · U3 · U4 (v1.9.1 refreshed)
NEXT:     P2b when device capacity — or next ops charter from feedback
```

---

## 1. Why this plan

H2 documented Windows-primary in-app updates because the Windows release job alone wrote `latest.json`. macOS/Linux already publish installer `.sig` files; they never make it into the updater manifest.

| Risk | Why it matters |
|------|----------------|
| Parallel `includeUpdaterJson` races | Lost platforms / `already_exists` failures |
| macOS/Linux installs never see in-app updates | Fall back to Releases forever |
| Docs still say Windows-primary after merge lands | Honesty drift |

**Maintainer constraint:** no physical CompanionShell (P2b parked). No new Path Truth features.

Deferred forever (unchanged): auto node rotation, AV certification, continuous packet capture, Obscur merge.

---

## 2. Tracks

| Band | Outcome |
|------|---------|
| **U1** | `scripts/build-updater-manifest.sh` builds `latest.json` from release `.sig` + installer URLs (Windows MSI/NSIS, Linux AppImage, macOS `.app.tar.gz`) |
| **U2** | Release workflow: all desktop jobs `includeUpdaterJson: false`; post-job single-writer uploads merged `latest.json` |
| **U3** | `verify-release-assets.sh` requires `windows-x86_64*`, `linux-x86_64*`, `darwin-aarch64`; docs drop Windows-primary claim |
| **U4** | Rebuild/upload `latest.json` on current share tag (`v1.9.1`) so live release matches the new shape |

---

## 3. Ship order (binding)

| Order | Band | Status |
|------:|------|--------|
| 1 | **U1** Manifest builder script | **Done** |
| 2 | **U2** Release workflow single-writer | **Done** |
| 3 | **U3** Verify + docs honesty | **Done** |
| 4 | **U4** Refresh `v1.9.1` latest.json | **Done** |

---

## 4. Success criteria

- [x] Builder produces platforms for Windows, Linux, and macOS from existing release assets  
- [x] Current share tag `latest.json` is not Windows-only  
- [x] `verify-release-assets.sh` fails if a desktop OS platform key is missing  
- [x] Docs match multi-OS in-app update (desktop only; mobile unchanged)  

---

## 5. Doc index

- [updater-signing.md](./updater-signing.md)  
- [product-plan-paste-release-trust-2026-11.md](./product-plan-paste-release-trust-2026-11.md) — prior  
- [github-ci.md](./github-ci.md) — release path  

_Last updated: 2026-11-02T02:20:00Z_
