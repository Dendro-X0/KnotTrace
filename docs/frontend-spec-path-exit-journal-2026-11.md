# Frontend spec — Path / exit journal (P2)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**API dependency:** `get_history_trends` + `active_path` on `HistoryTrendPoint` — [path-exit-journal-design.md](../specs/backend/path-exit-journal-design.md)  
**Surfaces:** Overview Quick teaser · Overview Advanced card

## Purpose

Compact recent path/exit history: active path, egress IP, slowdown shape, score grade — without opening raw DB or relying only on egress claim.

## Layout

### Quick

Full-width card under the bento grid (`PathExitJournal` `compact`). Max 3 rows. Empty: short “Run a few checks…” line (no primary Apply).

### Advanced

Card in the diagnosis column near Path break / Egress timeline. Max 8 rows. Same empty copy.

## Row presentation

One line per check, newest first:

`{time} · {grade} · {path|—} · {egress|—} · {shape|—}`

Use muted secondary text; no colored “safe” badges. Grade may use existing tone helpers if already used nearby.

## Wiring

| Piece | Path |
|-------|------|
| Mapper | `apps/desktop/src/lib/pathExitJournal.ts` |
| UI | `apps/desktop/src/components/PathExitJournal.tsx` |
| Consumer | `OverviewPage` (`state.trends`) |

## Acceptance

- [x] Quick shows ≤3 newest rows when trends exist  
- [x] Advanced shows ≤8  
- [x] Rows include grade + at least path or egress or shape when present on point  
- [x] Missing active_path does not invent VPN  
- [x] en + zh-Hans title/empty strings  
