# Frontend spec — Settings hub (Obscur-inspired)

Status: **shipped in tree**  
Builds on: [frontend-spec-settings-page-2026-11.md](./frontend-spec-settings-page-2026-11.md) · hub polish  

## Goal

Make Settings feel like a **hub preferences** surface (Obscur reference): secondary category rail + focused detail pane, current-value labels, per-card Reset — **teal KnotTrace**, not purple clone.

## Layout

```text
┌──────────────┬────────────────────────────────────┐
│ Search       │ Appearance                         │
│              │ Customize look & detail…           │
│ GENERAL      │ ┌ Language …………… Reset ┐          │
│  Appearance● │ │ CURRENT: EN          │          │
│  Monitoring  │ │ [ English | 中文 ]   │          │
│ TOOLS        │ └──────────────────────┘          │
│  Related     │ ┌ Theme … Reset ┐ …               │
│ ABOUT        │                                  │
│  About       │                                  │
└──────────────┴────────────────────────────────────┘
```

Narrow (`<lg`): horizontal category chips above content (no dual sidebar crush).

## Categories

| Id | Group | Content |
|----|-------|---------|
| `appearance` | General | Theme, density, language, reduced-motion toggle |
| `monitoring` | General | Background monitor |
| `related` | Tools | Links to Protect / Connect / DNS |
| `about` | About | Version, updates, tray note |

## Setting card pattern

- Title row + **Reset** (ghost) when non-default  
- Eyebrow: `CURRENT …` uppercase tracked  
- Control (segmented / switch / links)  
- Optional one-line help  

## Motion / a11y

- `data-reduce-motion="true"` on `<html>` when user opts in (also honor OS `prefers-reduced-motion`)  
- Active category: teal fill + soft hub glow (not purple)

## Non-goals

- Text scale / font size ladder  
- Profile / account / identity panes  
- New backend settings store  
- Copying Obscur purple tokens  
