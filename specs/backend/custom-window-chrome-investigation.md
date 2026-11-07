# Investigation — Custom window chrome (frameless title bar)

**Date (UTC):** 2026-11-07  
**Status:** Complete  
**Concern:** OS native title bar fights KnotTrace shell (screenshot: separate Windows chrome above dashboard)

## Observed

- `tauri.conf.json` windows entry uses default `decorations` (true) → OS title bar + app shell = two visual systems.
- Close already maps to **hide-to-tray** via `CloseRequested` + `prevent_close` in `lib.rs` (must preserve).
- No existing `TitleBar` / drag-region / window control UI; `@tauri-apps/api` window APIs unused in frontend.
- Capabilities `default.json` has `core:default` only — may need explicit window allow permissions for min/max/close/drag.

## Constraints

- Windows primary; macOS/Linux must still open and drag.
- CompanionShell (`?companion=1`) shares the same window label `main` → same chrome.
- Do not change tray semantics or monitor behavior.
- Specs-first UI: frontend spec before JSX.

## Options

| Option | Pros | Cons |
|--------|------|------|
| A. `decorations: false` + full-width custom title bar | Matches shell tokens; one surface | Need a11y labels + maximize state |
| B. Overlay titleBarStyle (macOS-centric) | Native traffic lights | Weak on Windows; still dual chrome |
| C. Transparent + CSS radius | Soft corners | WebView2 transparency bugs |

**Choice:** A — solid frameless + themed controls + `data-tauri-drag-region`.
