# Design — Protect review pass (one-click)

**Date (UTC):** 2026-11-07  
**Status:** Ready  
**Depends on:** [protect-review-pass-investigation.md](./protect-review-pass-investigation.md)

## Contract

| Input | Output |
|-------|--------|
| Current `ProtectAlert[]` + DNS/Connect recommend state | `ProtectReviewPassResult` |

### Result shape (frontend + optional invoke)

```ts
{
  applied: Array<{ kind: "dns" | "connect"; message: string; success: boolean }>;
  acknowledged: string[]; // alert titles marked reviewed this session
  skipped: string[];      // why an assist did not run
  note: string;           // one-line user summary
}
```

### Rules

1. **Explicit click only** — not background auto-protect; may apply DNS/Connect even when auto toggles are off (user consent = the click).  
2. **Never** mutate for integrity-only alerts (listeners, ARP, certs, forced-path *indicators*) — those are acknowledged only.  
3. **Opportunistic assists** — on click, also try DNS if a recommendation exists and Connect if `should_apply`, even when alert rows only say “View details”.  
4. **Acknowledge** uses sessionStorage key `knottrace-protect-ack` keyed by alert fingerprint for the app session.  
5. After successful apply → `runCheck()` once.  
6. Copy must say “reviewed / assists applied”, never “optimized / cleaned / secure”.

### Planner (`protectReviewPass.ts`)

- `assistKinds(alerts)` → unique `dns_assist` | `connect_assist` from alert actions  
- `acknowledgeTitles(alerts)` → titles with no mutating action (or only run_check)  
- Unit-tested

### UI

Primary button on trust card when visible (non-acked) alerts > 0: **Process review**.  
Disabled while running. Result line under the list.
