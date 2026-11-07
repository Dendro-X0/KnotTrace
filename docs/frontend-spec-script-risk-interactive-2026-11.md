# Frontend spec — Script risk interactive section

Status: **shipped in tree**  
Builds on: [frontend-spec-script-risk-scan.md](./frontend-spec-script-risk-scan.md) · [script-risk-scan-design.md](../specs/backend/script-risk-scan-design.md)

## Goal

Protect gains an **intuitive interactive section**: one primary control to start a bounded scan, clear results, and **honest next-step guidance** (review / reveal / copy) — still **indicators only**, never quarantine or kill.

## Composition

```text
┌─ Script risk ─────────────────────────────────────┐
│  Indicators only · short scope line               │
│  [ Allow & scan / Scan this PC ]   ← one primary  │
│  optional: also with health checks                │
├───────────────────────────────────────────────────┤
│  Results summary                                  │
│  Startup · Mining · Scripts on disk (hit cards)   │
│    per disk hit: Reveal · Copy path               │
├─ What to do next ─────────────────────────────────┤
│  1–3 guidance steps from hit categories           │
│  honesty footer (not AV / not cleaned)            │
└───────────────────────────────────────────────────┘
```

## Interaction

| State | Primary CTA |
|-------|-------------|
| No consent yet | **Allow & scan** (acks consent + runs scan) |
| Consent already | **Scan this PC** |
| Scanning | Disabled + Scanning… |

## Next steps (guidance only)

| Signal | Guidance |
|--------|----------|
| Disk hits | Reveal in folder / copy path — review before deleting anything yourself |
| Mining-like | Copy process name · open Task Manager yourself |
| Persistence | Review Startup apps / copy location |
| No hits | Re-scan later; empty ≠ safe |

## Non-goals

- Kill process / delete file / quarantine  
- “Malware cleaned” / “you are safe”  
- Full-disk AV  
