# Frontend spec — Overview stage focus

Status: **superseded for Quick** by [frontend-spec-overview-command-deck-2026-11.md](./frontend-spec-overview-command-deck-2026-11.md)  
Builds on: atmosphere/bento · hub polish · Path Truth tip / journal

## Goal

Make the **right-of-sidebar stage** (Quick Overview) read as one focused composition — less vertical sprawl, less empty hero air, evidence tiles stacked beside the status — without losing Path / Integrity / Export / journal.

## Problem (current)

- Hero is full-width and tall; metrics + CTA sit in leftover whitespace  
- Path / Integrity / Export sit in a wide uneven row under the hero  
- Journal floats alone with large gaps  

## Quick layout (`lg+`)

```text
┌─────────────────────────────┬──────────────────┐
│ Status (compact)            │ Path             │
│ headline · badges · summary │ Integrity        │
│ metrics · primary CTA       │ Export           │
├─────────────────────────────┴──────────────────┤
│ Path / exit journal (compact strip)            │
└────────────────────────────────────────────────┘
```

Narrow: stack tip → status → path → integrity → export → journal.

## Density rules

| Element | Change |
|---------|--------|
| Stage content | `max-w-5xl` centered in the stage column |
| StatusHero `emphasis="focus"` | Smaller type/icon; metrics + CTA in one denser block; less padding |
| Tip | One line + dismiss; tighter padding |
| Side teasers | `line-clamp` summaries; shared card density |
| Gaps | `gap-2.5` in focus grid (not airy `gap-3` stacks) |

## Non-goals

- Redesigning Advanced Overview stack  
- Removing Path Truth tip or journal  
- Neon/game chrome beyond existing hub tokens  
