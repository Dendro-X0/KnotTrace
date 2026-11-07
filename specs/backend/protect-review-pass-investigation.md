# Investigation — Protect one-click review pass

**Date (UTC):** 2026-11-07  
**Status:** Complete  

```text
HANDOFF ATOMIC STEP: none — greenfield (user request; path-exit charter complete)
PAUSED / CANCELLED:    none
CANONICAL OWNER:       protect + ProtectPage
PROOF BEFORE DONE:     L1 unit (planner) · desktop tsc build
SURFACE:               Tauri desktop
```

## Observed (screenshot)

Protect card lists e.g.:

1. Sites unreachable on current path  
2. Proxy path looks forced  
3. Open network service found (×2)

Badge: **N to review**. Per-row “Details” only expands copy; many rows have **empty** `actions` (integrity indicators).

## What can actually be “processed”

| Alert class | Source | Mutating fix? |
|-------------|--------|---------------|
| DNS / untrusted / degraded with assist actions | `alert_actions` → dns/connect | Yes — reversible Assist |
| Sites unreachable | often View details only | Connect only if API + recommendation |
| Integrity indicators (forced proxy, open listener, …) | `push_integrity_board_alerts` | **No** — observe-only |

## Product conflict if naive

A button labeled “Optimize all / Fix all” that claims to clear open listeners or forced-proxy evidence would violate observe-first / indicators-only / no AV.

## Chosen meaning of one-click

**Process review** = one explicit user gesture that:

1. Applies **eligible reversible assists** (DNS recommend → apply; Connect recommend → apply when `should_apply`).  
2. **Acknowledges** remaining indicator rows for this session (clears “to-do” queue without claiming fixed).  
3. Summarizes applied vs reviewed-only vs skipped.
