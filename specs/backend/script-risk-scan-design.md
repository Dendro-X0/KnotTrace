# Script risk scan — design

Product: [product-plan-script-risk-scan-2026-11.md](../../docs/product-plan-script-risk-scan-2026-11.md)

```text
HANDOFF ATOMIC STEP: none — greenfield
PAUSED / CANCELLED: full AV / signature DB / quarantine (Path Truth)
CANONICAL OWNER: network-core::suspicious_activity
PROOF BEFORE DONE: L1 unit · L2 cargo test suspicious · L3 Protect Run scan
```

## API

```text
pub fn run_script_risk_scan() -> SuspiciousActivitySnapshot
```

Always runs heuristics (caller gates consent). Settings still gate **health-check enrich**.

## Snapshot fields

Existing + `disk_hits: Vec<SuspiciousActivityHit>` (default `[]` for serde).

`source` becomes e.g. `hkcu_run+startup+user_folders+processes`.

## Disk roots (capped)

Windows: `%USERPROFILE%\Downloads`, `Desktop`, `%TEMP%`, `%LOCALAPPDATA%\Temp`  
Unix: `~/Downloads`, `~/Desktop`, `/tmp`, `$TMPDIR`  

Constraints: max depth 2, max 120 files examined, read first 4 KiB only, max 8 disk hits.

## File heuristic

Script extension in `{ps1,vbs,js,jse,wsf,bat,cmd}` **and** content/name matches elevated risk tokens (`-enc`, `downloadstring`, `http(s)://`, `FromBase64String`, `IEX`, `Invoke-Expression`, mining keywords).

Harmless `.ps1` without risk tokens → **not** a hit (reduces noise).

## Integrity

New kind `RiskyScriptFile` when disk hits present. Persistence / mining unchanged.

## UI

Protect **Script risk** card (Quick + Advanced): consent, enable-on-check switch, Run scan now, hit lists. Copy never says “malware found” or “clean.”
