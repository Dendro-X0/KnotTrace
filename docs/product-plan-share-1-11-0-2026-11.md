# Product plan — Share release v1.11.0

**Date (UTC):** 2026-11-07  
**Status:** **Tagged** (awaiting GitHub Release assets)  
**Repo:** KnotTrace (`network`) only  
**Depends on:** hub polish · Overview trust · pages harmony · script-risk interactive · custom window chrome

```text
GOAL:     Tag v1.11.0 so the hub UI surface (frameless chrome, Overview trust,
          cross-page harmony, interactive Script risk, search overlay fix) ships.
POSTURE:  Indicators only · not antivirus · no quarantine/kill · Path Truth
DONE:     V1 stamp · community-gate · commit/tag/push
NEXT:     After CI Release: verify-release-assets.sh v1.11.0 · stamp V3
```

_Gate note: community-gate green 2026-11-07; pushed `10a5a7f` + tag `v1.11.0`._

| Order | Band | Status |
|------:|------|--------|
| 1 | Version + CHANGELOG + docs stamp | **Done** |
| 2 | `bash scripts/community-gate.sh` | **Done** |
| 3 | Commit `Release v1.11.0` · tag · push | **Done** (`10a5a7f` · `v1.11.0`) |
| 4 | `verify-release-assets.sh v1.11.0` · stamp | Pending after GitHub Release |

### Ships in this release

- Custom frameless window chrome (TitleBar)
- Hub polish (rail / stage / motion / CTA glow)
- Overview score hero + Path/Protect trust reconciliation
- Cross-page hub harmony (Network / DNS / Connect / Protect)
- Settings hub layout
- Protect Process review (honest assists + ack)
- Script risk interactive (Allow & scan → results → next steps)
- Global search overlay containment fix

_Last updated: 2026-11-07T12:30:00Z_
