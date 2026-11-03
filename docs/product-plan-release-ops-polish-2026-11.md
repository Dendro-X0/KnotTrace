# Product plan — Release ops polish

**Date (UTC):** 2026-11-02  
**Status:** **Complete** (successor to multi-OS updater; P2b unchanged)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Multi-OS updater complete — [product-plan-multi-os-updater-2026-11.md](./product-plan-multi-os-updater-2026-11.md)

```text
GOAL:     Keep release/share scripts reliable offline and honest in docs —
          LF shells, fixture-tested updater merge, plan NEXT pointers current.
POSTURE:  Observe-first · claim discipline · no phone readiness · no fake Mbps
BASELINE: v1.9.1 multi-OS latest.json + community gate
DONE:     Charter · O1 · O2 · O3 · O4
NEXT:     P2b when device capacity — wait for community feedback otherwise
```

---

## 1. Why this plan

Multi-OS updater landed, but ops edges remain:

| Risk | Why it matters |
|------|----------------|
| CRLF shebangs on Windows checkouts | CI `Permission denied` / bad interpreter (already hit once) |
| Updater merge only tested live via `gh` | Regressions wait for a tag |
| Stale plan `NEXT` pointers | Agents/maintainers chase finished charters |

**Maintainer constraint:** P2b physical CompanionShell still parked. No Path Truth feature bands.

Deferred forever (unchanged): auto node rotation, AV certification, continuous packet capture, Obscur merge.

---

## 2. Tracks

| Band | Outcome |
|------|---------|
| **O1** | `.gitattributes` forces `*.sh` LF; normalize existing gate/updater scripts |
| **O2** | `build-updater-manifest.sh --from-dir` builds from local sig fixtures (no network) |
| **O3** | `community-gate.sh` always runs O2 fixture; CI core job runs the same cheap check |
| **O4** | CHANGELOG Unreleased + stamp prior plans’ NEXT to this charter / complete |

---

## 3. Ship order (binding)

| Order | Band | Status |
|------:|------|--------|
| 1 | **O1** LF + gitattributes | **Done** |
| 2 | **O2** Offline `--from-dir` | **Done** |
| 3 | **O3** Gate + CI hook | **Done** |
| 4 | **O4** Changelog + NEXT sync | **Done** |

---

## 4. Success criteria

- [x] `*.sh` checked out as LF on fresh clones  
- [x] Offline updater fixture produces multi-OS platforms without `gh`  
- [x] Community gate / CI fails if fixture merge breaks  
- [x] Plan NEXT pointers no longer claim unfinished multi-OS updater  

---

## 5. Doc index

- [product-plan-multi-os-updater-2026-11.md](./product-plan-multi-os-updater-2026-11.md) — prior  
- [development.md](./development.md) — gates  
- [updater-signing.md](./updater-signing.md)  

_Last updated: 2026-11-02T02:25:00Z_
