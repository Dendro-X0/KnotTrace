# Path / exit journal (P2) — design

**Date (UTC):** 2026-11-06  
**Status:** Implemented (L1 unit + L2 reliability-gate green)  
**Charter:** [product-plan-path-exit-clarity-2026-11.md](../../docs/product-plan-path-exit-clarity-2026-11.md) band **P2**  
**Owner:** `HistoryTrendPoint` projection + `pathExitJournal` mapper + Overview UI  
**Depends on:** existing history store · path-break on reports  
**Out of scope:** new probes · rewriting TrendCharts · egress timeline claim changes

## Problem

Recent path/exit context is split across score trends and egress claim — not one compact journal of path + IP + shape + grade.

## Goal

Show a short recent history list so users can answer: “what path was I on, which exit IP, what shape/grade did checks report?”

## Backend (projection only)

Extend `HistoryTrendPoint`:

```text
active_path: Option<PathBreakPath>  // from path_break.active_path
```

`report_to_trend_point`: copy when `path_break` present. No DB migration.

## Frontend mapper

Module: `apps/desktop/src/lib/pathExitJournal.ts`

Input: `HistoryTrendPoint[]` (chronological or newest-last as returned today).

Output:

```text
PathExitJournal
├── rows: PathExitJournalRow[]   # newest first, cap Quick=3 / Advanced=8
│   ├── at: ISO timestamp
│   ├── grade
│   ├── activePath: string | null   # snake label or null
│   ├── egressIp: string | null
│   ├── slowdownShape: string | null
│   └── summaryLine: string         # preformatted for tests / plain fallback
└── empty: boolean
```

Rules:

- Prefer newest `limit` points with at least one of: grade, shape, egress, path.
- Do not invent Mbps or “safe/unmonitored.”
- Missing `active_path` on legacy rows → show as unknown / omit path token (do not fake VPN from proxy alone).

## UI

| Surface | Component | Behavior |
|---------|-----------|----------|
| Overview Quick | `PathExitJournal` compact | Title + up to 3 rows under bento; empty → one calm line |
| Overview Advanced | same, non-compact | Up to 8 rows near Path / Egress; no duplicate Run-check CTA |

Network page: **not required** this band (Overview is enough).

## Copy

i18n keys under `pathJournal.*` (en + zh-Hans): title, empty, column hints optional. Row body can stay structured (grade · path · IP · shape) with localized shape/path labels via existing `formatTag` / small key map.

## Subtraction

- Do not replace EgressTimelinePanel — journal is row history; timeline stays claim.  
- Do not add a third Overview hero CTA.  
- Do not change diagnosis ranking.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Rust: trend point includes active_path when path_break set. TS: mapper caps + newest-first + null path handling |
| L2 | `bash scripts/reliability-gate.sh` |
| L3 | Optional: run 2–3 checks, confirm journal rows match Overview |

## Done when

- [x] Specs + frontend note  
- [x] `active_path` on trend points  
- [x] Mapper + Overview Quick/Advanced wired  
- [x] L1 + L2 green  
