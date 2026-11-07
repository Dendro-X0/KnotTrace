# Frontend spec — Hub polish (game-hub feel)

Status: **shipped in tree** (H1–H4 shell pass)  
Builds on: custom window chrome · atmosphere/bento · Quick/Advanced shell

## Goal

Make KnotTrace feel like a **composed desktop hub** — seamless frameless chrome, deeper stage, fluid nav/page motion — without Steam neon kitsch or purple glow clones.

## Direction

| Do | Don’t |
|----|--------|
| One continuous shell (title → rail → stage) | Separate OS chrome + bolted strip |
| Teal rim light / soft primary glow on active rail + CTAs | Purple/indigo neon, particle fog |
| Page enter: fade + slight scale; nav active: soft glow + 220ms ease | Parallax, bounce springs, constant pulse noise |
| Stage panel as inset “viewport” with rim | Flat equal-gray cards only |
| Honor `prefers-reduced-motion` | Motion that fights WebView2 (no card `translateY`) |

## Layout

```text
┌─────────────────────────────────────────────┐
│ TitleBar (drag + window controls)           │
├──────────┬──────────────────────────────────┤
│ Hub rail │ Stage (rounded inset panel)      │
│ brand    │ header · page enter · cards      │
│ search   │                                  │
│ nav      │                                  │
└──────────┴──────────────────────────────────┘
```

## Bands

| Band | Outcome |
|------|---------|
| **H1** | Shell unity — hub title bar + stage rim; chrome already frameless |
| **H2** | Rail interactivity — nav hover/active glow, search affordance |
| **H3** | Atmosphere — stronger shell radials, stage edge light |
| **H4** | Fluid motion — page enter, stagger, CTA soft lift/glow |

## Non-goals

- Rewriting page copy or diagnosis logic  
- Glassmorphism stacks / acrylic blur everywhere  
- Animated backgrounds or audio  
- Moving Settings into the title bar  
