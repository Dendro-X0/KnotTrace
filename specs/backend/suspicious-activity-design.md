# Suspicious activity indicators (surveillance / mining heuristics)

Status: **implemented** (Bands I1–I5)  
Product plan: [product-plan-suspicious-activity-indicators-2026-11.md](../../docs/product-plan-suspicious-activity-indicators-2026-11.md)

## Goal

Opt-in heuristics that surface **indicators** consistent with script-host persistence (monitoring-like) or mining-like processes. Never a malware verdict; never kill/quarantine.

## Non-goals

- AV signature engines / reputation cloud  
- Continuous background scanning  
- Disabling startup items or terminating processes  
- Guaranteeing detection of all surveillance or miners  

## Canonical owner

| Piece | Owner |
|-------|--------|
| Scan + settings | `network-core::suspicious_activity` |
| Enrich | desktop `monitor::enrich_history_claims` after autoruns |
| UI toggle | Protect page |
| Indicator kinds | `ScriptHostPersistence`, `MiningLikeProcess` |

## Settings

`suspicious_activity_settings.json`: `{ enabled, consent_ack }` — both required to scan.  
Consent: `knottrace.suspicious_activity.consent.v1`.

## Snapshot

```text
SuspiciousActivitySnapshot {
  scanned, enabled, source,
  persistence_hits: [{ name, location, detail? }],
  mining_hits: [{ name, location, detail? }],
  summary
}
```

Hits capped at 8 per category.

## Heuristics (v1)

### Script-host persistence (`script_host_persistence`)

Windows-first sources:

- `HKCU\…\Run` values whose command invokes `powershell` / `pwsh` / `wscript` / `cscript` / `mshta` (optional: `cmd /c` launching those)
- User Startup folder script files (`.ps1`, `.vbs`, `.js`, `.jse`, `.wsf`, `.bat`, `.cmd`)

Raise confidence when path/command references `%TEMP%`, Downloads, or encoded command (`-enc` / `-encodedcommand`) / remote URL.

### Mining-like process (`mining_like_process`)

Windows: sample `Win32_Process` Name / ExecutablePath / CommandLine (capped).

Match if:

- Name/cmdline keywords: `xmrig`, `ethminer`, `minerd`, `nicehash`, `stratum`, `hashrate`, …  
- Or executable under Temp/Downloads **and** cmdline mentions pool-like tokens  

Linux/macOS: best-effort `/proc` or `ps` sample with same keyword rules; document as partial in platform-support.

## Claim copy

Indicator messages must include “indicator only, not malware detection.”  
Integrity disclaimer unchanged; COMMUNITY remains “not antivirus.”

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit tests for `looks_script_host_command` / `looks_mining_like` |
| L2 | `cargo test -p network-core suspicious` |
| L3 | Enable on Protect → check → integrity shows hit when fixture-like Run entry exists (maintainer) |
