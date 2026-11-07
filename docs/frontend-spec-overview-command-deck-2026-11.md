# Frontend spec — Overview command deck

Status: **superseded for Quick** by [frontend-spec-overview-hero-sections-2026-11.md](./frontend-spec-overview-hero-sections-2026-11.md)  
Supersedes Quick layout in [frontend-spec-overview-focus-2026-11.md](./frontend-spec-overview-focus-2026-11.md) for composition only.

## Why

The focus split (tall status | stacked teasers + `max-w-5xl`) left empty stage gutters and a hollow center. Need a **different style**: one full-width deck, not a floating island.

## Quick composition

```text
┌─────────────────────────────────────────────────────┐
│ tip (optional thin)                                 │
├─────────────────────────────────────────────────────┤
│ STATUS RAIL                                         │
│ icon · headline · badges · summary                  │
│ metrics ………………… primary CTA                        │
├──────────────┬──────────────┬───────────────────────┤
│ Path         │ Integrity    │ Export                │
├──────────────┴──────────────┴───────────────────────┤
│ Path / exit journal                                 │
└─────────────────────────────────────────────────────┘
```

- Full stage width (no `max-w-5xl`)  
- One deck surface (`overview-deck`) with shared border; inner cells use hairline dividers  
- Status uses `emphasis="rail"` — horizontal, short, no tall hero air  
- Evidence tiles equal thirds on `md+`  

## Non-goals

- Advanced Overview rewrite  
- Purple neon / glass stacks  
