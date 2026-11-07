# Frontend spec — Custom window chrome

Status: **shipped in tree**  
Builds on: atmosphere shell tokens · Sidebar brand · hide-to-tray close

## Goal

Remove the stock OS title bar so the window reads as one KnotTrace surface — same shell gradient / teal–slate language as the dashboard.

## Layout

```text
┌──────────────────────────────────────────────────┐
│ ● KnotTrace                    [─] [□] [×]       │  TitleBar (drag)
├────────────┬─────────────────────────────────────┤
│  Sidebar   │  surface-panel (unchanged)          │
│            │                                     │
└────────────┴─────────────────────────────────────┘
```

- Height: `h-9` (36px); hairline `border-b border-border/50`.
- Background: transparent over body `shell-gradient` (no second panel color).
- Left: 20px app icon + `brand.name` (muted, small) — drag region.
- Right: three icon buttons (Minimize / Maximize|Restore / Close) — not drag.
- Close hover: destructive tint; min/max: muted → foreground.
- Maximized: swap Maximize icon for Restore; still one control.
- CompanionShell: same TitleBar above the narrow column.

## Copy (i18n)

| Key | en | zh-Hans |
|-----|----|---------|
| `chrome.minimize` | Minimize | 最小化 |
| `chrome.maximize` | Maximize | 最大化 |
| `chrome.restore` | Restore | 还原 |
| `chrome.close` | Close | 关闭 |
| `chrome.dragHint` | Drag to move window | 拖动以移动窗口 |

## Non-goals

- Moving Settings/theme into the title bar  
- macOS-only left traffic-light reposition (Windows-primary; controls stay right)  
- Animated maximize transitions  
