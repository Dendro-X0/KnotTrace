# L3 device / multi-OS proof (Band P2)

Community-readiness charter splits P2 so automation can land without maintainer hands on a phone.

| Slice | What | Status |
|-------|------|--------|
| **P2a** | Headless `network-probe` L3a on Linux + macOS + Windows CI runners | Automated via CI job `Headless probe` |
| **P2b** | Physical / emulator CompanionShell (check + export) | **Parked** until device capacity |

## P2a — already gated in CI

```bash
./scripts/headless-regression.sh
```

CI (`.github/workflows/ci.yml`) runs the same script on `ubuntu-latest`, `macos-latest`, and `windows-latest` (bash shell; one retry on egress flake). Green matrix cells mean parse + `score` + `integrity.disclaimer` + `path_break` on those OSes.

## P2b — physical CompanionShell checklist (when capacity returns)

Do **not** flip [platform-support.md](./platform-support.md) mobile cells to “ready” until every item below is recorded in release notes.

1. Configure mobile signing secrets ([mobile-signing.md](./mobile-signing.md)) so Release publishes APK/IPA (today those jobs skip).
2. Install signed APK (or iOS build) on a real device or emulator.
3. Cold start → run health check → confirm grade / path-break / integrity render.
4. Export diagnostic JSON; confirm `schema_version: knottrace.diagnostic_bundle.v1`.
5. Note OS version, app version, and date in CHANGELOG + platform-support honesty line.
6. Optionally repeat desktop CompanionShell preview (`?companion=1`) click-smoke on Windows.

Related: [frontend-spec-mobile-companion.md](./frontend-spec-mobile-companion.md) B2 acceptance.
