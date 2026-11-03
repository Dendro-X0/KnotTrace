# Product plan — Share release v1.10.0

**Date (UTC):** 2026-11-03  
**Status:** **Ready to tag** (local gates green)  
**Repo:** KnotTrace (`network`) only  
**Depends on:** UI density · script-risk scan · Settings/i18n · atmosphere/Protect calm

```text
GOAL:     Tag v1.10.0 so consumer UX + bounded script-risk ship in installers.
POSTURE:  Indicators only · not antivirus · P2b parked · CompanionShell desktop preview
DONE:     Charter · V1 · community-gate.sh
NEXT:     Commit · tag v1.10.0 · push tag · V3 verify-release-assets.sh v1.10.0
```

| Order | Band | Status |
|------:|------|--------|
| 1 | Version + CHANGELOG + docs stamp | **Done** |
| 2 | `bash scripts/community-gate.sh` | **Done** (R1 + R4 + H3 + O3) |
| 3 | Commit `Release v1.10.0` · tag · push | Pending |
| 4 | `verify-release-assets.sh v1.10.0` · stamp | Pending after GitHub Release |

### Ships in this release

- Quick / Advanced density (default Quick)
- Settings page + en / zh-Hans locales
- Atmosphere / bento Overview + theme refresh
- Protect consumer-calm trust hero
- Bounded **Script risk** scan (not AV)
- Card border hairline clip fix (WebView2)

_Last updated: 2026-11-03T12:40:00Z_
