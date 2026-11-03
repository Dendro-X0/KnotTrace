# Frontend spec — Settings page

Status: **shipped in tree**  
Pairs with: [frontend-spec-consumer-calm-protect-2026-11.md](./frontend-spec-consumer-calm-protect-2026-11.md)

## Goal

A dedicated **Settings** area for average users: appearance, monitoring, about/updates — calm grouped rows, not another diagnostic dashboard.

## Route

- `PageId`: `settings`
- Sidebar: bottom item (gear), separate from job pages
- Global search: “Settings”, theme, density, monitor, updates
- AppHeader title: Settings (no redundant FeaturePage H1)

## Sections

| Section | Contents |
|---------|----------|
| Appearance | Theme, Density (Quick/Advanced), **Language** (JSON locales) with one-line help |
| Monitoring | Background monitor switch; poll interval as read-only fact |
| Shortcuts | Links to Protect (alerts/script risk) and Connect (API) |
| About | App version, tray note, Check for updates |

## Sidebar chrome

Remove Theme/Density toggles from sidebar footer (canonical home is Settings). Keep version line.

## Non-goals

- New backend settings store
- Moving Script risk off Protect
- Rebuilding CompanionShell
