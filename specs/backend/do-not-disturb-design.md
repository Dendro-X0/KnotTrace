# Do Not Disturb mode — design (v1.4.x)

## Goal

Allow KnotTrace to keep monitoring and optional auto-protect actions while suppressing **all system notifications**.

## Non-goals

- Does not disable health checks, tray status, in-app Protect alerts, or auto DNS/proxy actions
- Does not replace fine-grained notify toggles; those still apply when DND is off

## Contract

| Field | Type | Default | Persist |
|-------|------|---------|---------|
| `do_not_disturb` | `bool` on `ProtectSettings` | `false` | `protect_settings.json` via serde `default` |

### Behavior

1. If `do_not_disturb` is `true`, `should_notify(...)` returns `None` (before other notify rules).
2. Automated-check warrant gating is irrelevant when DND is on — no notification is shown.
3. Existing `notify_on_*` toggles are unchanged when DND is off.
4. UI: Protect page toggle labeled **Do Not Disturb** with hint that KnotTrace stays silent (no OS notifications).

## Proof

| Layer | Command / check |
|-------|-----------------|
| L1 | Unit test: DND on → `should_notify` is `None` even for Poor / untrusted |
| L2 | `cargo test -p network-core protect::` |
| L3 | TypeScript `ProtectSettings` includes field; Protect page toggle wired |
