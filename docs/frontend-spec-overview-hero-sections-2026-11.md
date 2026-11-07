# Frontend spec — Overview hero + categorized sections

Status: **shipped in tree**  
Inspired by focused “boost” apps’ **hero → categories** composition — **not** their Mbps / red / neon kitsch.  
Supersedes Quick composition in command-deck / focus specs.

## Goal

Quick Overview: one **visual hero** (headline + score ring + equal metric strip), **CTA below the hero**, then **categorized sections** (Path, Integrity, Export) with matched chrome, plus a quiet latest-exit line.  
Trust / attention overrides when Path or Integrity fire on a good score: see `frontend-spec-overview-trust-2026-11.md`.

## Composition

```text
[ tip — quiet banner ]

┌────────────── Hero ──────────────────────────┐
│           Looking good · badges              │
│           one-line summary                   │
│                 ╭────╮                       │
│                 │ 92 │                       │
│                 │/100│                       │
│                 ╰────╯                       │
│        [Internet] [Gateway] [DNS]            │
└──────────────────────────────────────────────┘
              [ primary CTA ]

┌ Path ──┐ ┌ Integrity ─┐ ┌ Export ──┐
│ matched uppercase section chrome            │
└────────┘ └────────────┘ └──────────┘

┌ Path / exit journal ─────────────────────────┐
└──────────────────────────────────────────────┘
```

## Rules

| Do | Don’t |
|----|--------|
| Score out of 100 (honest health) | Fake “+24% PERFORMANCE” |
| Teal/emerald arc by grade | Red booster chrome |
| Soft ring glow (card `overflow-visible`) | Angular bevel / hardware vents |
| Equal metric chips under the ring | Asymmetric flank columns with empty right |
| CTA centered under the hero card | CTA jammed into a metric column |
| Matched category headers (icon + uppercase label) | Export card with oversized icon tile |
| Full stage width (no centered `max-w` island) | Narrow column with empty side gutters |

## Non-goals

- Claiming boost/optimize  
- Copying Smart Game Booster branding  
- Advanced Overview rewrite  
