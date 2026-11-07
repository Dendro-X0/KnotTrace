# Frontend spec — Overview trust / information hierarchy

Status: **shipped in tree**  
Builds on: overview hero + categorized sections · next-step plan

## Goal

Quick Overview must tell **one coherent story** in the first five seconds: measured check score stays honest, but the hero never says “Looking good” when Path or Integrity evidence needs attention.

## Rules

| Do | Don’t |
|----|--------|
| Keep numeric score from the check (no fake regrade) | Change 100 → 60 to “match” path cards |
| Override headline + summary when path break or integrity signals fire | Keep “Looking good” + “Connection looks healthy” above a failed path |
| One primary teal CTA (next-step) when attention surfaces; demote header Run check | Two equal `hub-cta` buttons |
| Quiet tip + lighter journal in Quick | Tip competing with hero; three identical journal rows as peers |
| Human badge labels (“System proxy”, “8 signals”) | Shouty `SYSTEM PROXY` / `INDICATOR(S)` |

## Hero attention (good score + evidence)

Priority when `score.grade === good`:

1. Path break (`break_hop` not `none`) **and** integrity count > 0 → headline + summary for both  
2. Path break only → path attention copy  
3. Integrity only → Protect attention copy  
4. Else → existing grade headline + calm summary

Special-case hero keys (captive, bufferbloat, tunnel, partial site, …) still win over this layer when they already fire in `buildQuickHeroSummary`.

## Non-goals

- Backend score formula rewrite  
- Advanced Overview layout change  
- Removing Path / Integrity cards (they stay the evidence band)  
