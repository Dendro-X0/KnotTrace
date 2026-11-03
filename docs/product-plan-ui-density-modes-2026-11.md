# Product plan — UI density: Quick vs Advanced

**Date (UTC):** 2026-11-03  
**Status:** **Complete**  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer  
**Depends on:** v1.9.3 Path Truth + integrity + suspicious-activity indicators  
**Spec:** [frontend-spec-ui-density-modes.md](./frontend-spec-ui-density-modes.md)

```text
GOAL:     Make KnotTrace feel modern and calm for non-experts while keeping
          full Path Truth depth one toggle away — without fake “safe” claims.
POSTURE:  Observe-first · one primary action · progressive disclosure
BASELINE: Dense Overview + Network cards
DONE:     Charter · U1 · U2 · U3 · U4 · U5
NEXT:     Optional share patch when ready — P2b still parked
```

---

## 1. Modes (binding)

| Mode | Audience | First viewport |
|------|----------|----------------|
| **Quick** (default) | Non-technical | Score + next steps + Path/Integrity teasers; Network = probes + link + local repair |
| **Advanced** | Power users | Full Overview / Network depth |

Toggle in sidebar (with theme); persist `knottrace-ui-density`.

## 2. Ship order

| Order | Band | Status |
|------:|------|--------|
| 1 | **U1** Density spec | **Done** |
| 2 | **U2** Mode setting + chrome toggle | **Done** |
| 3 | **U3** Overview Quick | **Done** |
| 4 | **U4** Network Quick | **Done** |
| 5 | **U5** Docs | **Done** |

## 3. Success criteria

- [x] New user on Quick sees no benchmark form above the fold  
- [x] Advanced restores prior Overview/Network surfaces  
- [x] COMMUNITY / START-HERE mention Quick default  
- [x] Mode survives restart (`localStorage`)  

_Last updated: 2026-11-03T05:45:00Z_
