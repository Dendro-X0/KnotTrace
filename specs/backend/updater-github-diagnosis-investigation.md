# Investigation: updater UI broken + GitHub connectivity under-diagnosed

**Date:** 2026-08-05  
**Surface:** Tauri desktop (`apps/desktop`) + `network-core` diagnosis  
**Status:** Root causes identified; design in sibling doc

## Symptoms (user + screenshots)

1. Banner shows `Update available: v` and `You are on vundefined`.
2. **Get update** click does nothing.
3. In-app auto-update feels broken (no install path).
4. App does not clearly diagnose GitHub HTTPS failures caused by proxy/VPN (user also saw `git push` fail: `Could not connect to github.com port 443`).

## Evidence

### A. IPC field rename mismatch (confirmed)

`apps/desktop/src-tauri/src/updater.rs`:

```rust
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck { /* current_version, latest_version, release_url, can_install_in_app */ }
```

Frontend `UpdateCheck` / `UpdateBanner` / `openUpdateRelease` read **snake_case** (`latest_version`, `current_version`, `release_url`, `can_install_in_app`).

Runtime effect:

| Rust JSON key | UI reads | Result |
|---|---|---|
| `latestVersion` | `latest_version` | `undefined` → `"vundefined"` |
| `currentVersion` | `current_version` | `undefined` → `"vundefined"` |
| `canInstallInApp` | `can_install_in_app` | falsy → always **Get update**, never **Install update** |
| `releaseUrl` | `release_url` | falsy → `openUpdateRelease` early-returns with no feedback |

This alone explains display + dead button + “auto-update doesn’t work” for users who should get signed in-app install.

`AppInfo` has no rename and single-word fields — header version from `get_app_info` is unrelated to the banner bug.

### B. Signed path omits release URL

When `app.updater().check()` succeeds, `release_url` is always `None`. Even after casing fix, a forced fallback UI would need a default GitHub URL. Prefer showing **Install update** when `can_install_in_app` is true.

### C. Release channel itself is healthy

`https://github.com/Dendro-X0/KnotTrace/releases/latest/download/latest.json` serves v1.7.0 with Windows MSI + signatures. Tag `v1.7.0` assets include `latest.json` and `*.sig`. Channel/config is not the primary UI bug.

### D. Updater HTTP client ignores KnotTrace proxy detection

`fallback_github_check` builds a bare `reqwest::Client` (no system proxy from `network-core` env). Health probes use detected Windows/system proxy. Environments that need a proxy to reach GitHub can fail update checks while other app probes succeed (or the reverse), with no user-visible update error (`checkForUpdates` swallows errors → `setUpdateCheck(null)`).

### E. GitHub under-diagnosis (confirmed gap)

1. `site_access_degraded` requires **majority** failures (`failure_count * 2 >= checked_domains`). A lone `github.com` failure among 4 verification domains does **not** emit SiteAccess hints.
2. `probe_proxy_path_report` runs only when `environment.proxy.enabled`. VPN-without-HTTP-proxy users get no proxy-vs-direct GitHub compare.
3. Sensitive prepend (`github.com`, `api.github.com`, …) only when proxy is enabled. Default DNS verification list includes `github.com` but not `api.github.com`.
4. Proxy-path summaries can mention GitHub as an example string, but there is no dedicated “GitHub / git HTTPS blocked” bottleneck with actionable VPN/proxy copy.

## Failed / rejected hypotheses

| Hypothesis | Verdict |
|---|---|
| `latest.json` missing / unsigned → empty version strings | Rejected for UI `undefined` (JS property miss). Channel verified OK. |
| `get_app_info` returns undefined version | Unrelated; banner uses `UpdateCheck.current_version`. |
| Tauri updater completely broken | Secondary; UI never offers Install because of casing. |

## Canonical owners

| Concern | Owner |
|---|---|
| Update IPC + install commands | `apps/desktop/src-tauri/src/updater.rs` |
| Banner / open release | `UpdateBanner.tsx`, `useCompanion.ts` |
| GitHub/proxy diagnosis | `crates/network-core/src/diagnosis.rs` (+ small reachability target tweak) |

## Proof plan (after fix)

- **L1:** unit test `UpdateCheck` JSON uses snake_case keys; diagnosis unit test for GitHub-only failure; `cargo test -p network-core --lib`; `cargo test -p network-desktop updater`; `tsc --noEmit`
- **L2:** mental: signed update → Install path; fallback → Get update opens releases/latest
- **L3 (manual):** run packaged or `tauri dev`, confirm banner versions; with proxy/VPN blocking GitHub, health check shows GitHub-specific hint
