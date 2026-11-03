# Product plan — Suspicious activity indicators (surveillance / mining heuristics)

**Date (UTC):** 2026-11-03  
**Status:** **Complete** (P2b / AV still out of scope)  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** Integrity board + autoruns + process↔egress at v1.9.2  
**Spec:** [suspicious-activity-design.md](../specs/backend/suspicious-activity-design.md)

```text
GOAL:     Surface opt-in indicators that *may* be consistent with unknown
          surveillance tooling or cryptomining on this machine — without
          becoming antivirus or claiming “clean / unmonitored.”
POSTURE:  Observe-first · indicators with confidence · reversible / no kill
BASELINE: Integrity board, listeners, autoruns, process↔egress @ v1.9.2
DONE:     Charter · I1 · I2 · I3 · I4 · I5
NEXT:     Shipped @ **v1.9.3** — P2b still parked
```

---

## 1. Why this plan

Users do introduce unwanted software via downloads. KnotTrace already helps on **path / exposure**. Adjacent job: flag **local activity patterns** that look like script-host persistence or mining — as heuristics only.

## 2. Claim rules (binding)

| Do | Do not |
|----|--------|
| “Indicators consistent with …” | “Malware found / virus cleaned” |
| Opt-in, consent-gated scans | Silent background AV-style scanning |
| Cap hits; export via integrity indicators | Quarantine, kill, disable autoruns |
| Keep COMMUNITY “not antivirus” | AV/EDR certification language |

**Deferred forever (unchanged):** full AV engine, signature DBs, continuous packet capture, Obscur merge, auto node rotation.

## 3. Ship order

| Order | Band | Status |
|------:|------|--------|
| 1 | **I1** Design spec | **Done** |
| 2 | **I2** Script-host persistence indicators | **Done** |
| 3 | **I3** Mining-like process indicators | **Done** |
| 4 | **I4** UI + consent + enrich | **Done** |
| 5 | **I5** Docs + unit tests | **Done** |

## 4. Success criteria

- [x] Opt-in scan unit tests for script-host / mining heuristics  
- [x] UI never says “virus” / “cleaned” / “you are safe”  
- [x] COMMUNITY still states not antivirus  
- [x] Integrity kinds `script_host_persistence` / `mining_like_process` when hits present  

_Last updated: 2026-11-03T05:10:00Z_
