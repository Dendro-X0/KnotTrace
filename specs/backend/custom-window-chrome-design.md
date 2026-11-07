# Design — Custom window chrome

**Date (UTC):** 2026-11-07  
**Status:** Ready  
**Depends on:** [custom-window-chrome-investigation.md](./custom-window-chrome-investigation.md)

## Contract

| Surface | Behavior |
|---------|----------|
| Window | `decorations: false`, `shadow: true` on `main` |
| Title bar | Full-width row (~36px), shell background, brand left, controls right |
| Drag | Most of bar is `data-tauri-drag-region`; controls are not |
| Minimize | `getCurrentWindow().minimize()` |
| Maximize | `toggleMaximize()`; double-click drag region toggles |
| Close | `close()` → existing Rust `CloseRequested` → hide to tray (unchanged) |
| Non-Tauri | Title bar still renders; window actions no-op (Vite browser) |

## Permissions (`capabilities/default.json`)

Allow: minimize, maximize, unmaximize, toggleMaximize, close, isMaximized, startDragging.

## Proof

| Layer | Proof |
|-------|--------|
| L1 | `npm run build` (tsc) in `apps/desktop` |
| L2 | Manual: drag · min · max · close-to-tray · theme light/dark |
| Non-goal | No community-gate required for chrome-only polish |

## Non-goals

- Acrylic/Mica blur  
- Custom resize grips beyond OS default for undecorated windows  
- Changing tray menu or quit path  
