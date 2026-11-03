# Frontend spec — Atmosphere, bento Overview, motion

Status: **shipped in tree**  
Builds on: Quick Overview hero, Settings, consumer-calm Protect

## Goal

Give KnotTrace a more **composed, artistic Overview** (brick-wall / bento tiles), richer **light & dark** atmosphere, and **subtle motion** — without becoming a purple splash clone or a finance-dashboard pastiche.

## Direction

| Do | Don’t |
|----|--------|
| Teal–sky companion accents on deep slate / soft paper | Purple/indigo glow themes |
| Asymmetric bento: hero wide, Path/Integrity/Export as uneven tiles | Uniform equal-height card stack only |
| Soft fade/slide on page change; light card hover lift | Loud parallax, neon pulses, spinner splash |

## Overview Quick layout (12-col)

```text
[=========== Status hero (12) ===========]
[ Path (5–6) ][ Integrity (4–5) ][ Export (3–12) ]
```

Narrow: stack; `md+`: Path | Integrity; Export full or side tile; `xl`: three-up with Export as a shorter side brick.

## Theme tokens

Refresh `--shell-gradient`, `--panel-gradient`, `--card-gradient` for light (paper + cool sky) and dark (ink + teal rim light). Keep semantic status greens/ambers/roses.

## Motion

- `.motion-page-enter` on main when `page` changes  
- `.motion-stagger` children 40–80ms delay  
- Card hover: 1px lift + border brighten (`prefers-reduced-motion: reduce` disables)

## Non-goals

- Full masonry JS library  
- Rewriting every Advanced panel  
- New brand mark / splash screen  
