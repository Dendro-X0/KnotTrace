# Design: updater IPC fix + GitHub access diagnosis

**Depends on:** [updater-github-diagnosis-investigation.md](updater-github-diagnosis-investigation.md)  
**Non-goals:** Authenticode; changing minisign keys; diagnosing non-GitHub git hosts; fake “boost GitHub speed” claims.

## 1. Updater IPC (desktop)

### Contract

Serialize `UpdateCheck` and `UpdateProgressEvent` with **snake_case** field names (match `network-core` IPC and existing TS types). Remove `rename_all = "camelCase"`.

Always populate `release_url` when an update is available:

- Signed path: `https://github.com/Dendro-X0/KnotTrace/releases/tag/v{version}` (or `/releases/latest` if tag unknown)
- Fallback GitHub API: keep `html_url` from the release payload

### Fallback check + proxy

`fallback_github_check` should prefer the same system proxy KnotTrace already detected (pass optional proxy server from a thin helper, or call `network_core` env snapshot). Timeout stays ~12s. On failure, return a clear `Err` string (already done); frontend must not swallow silently.

### Frontend

- `openUpdateRelease`: if `release_url` missing, open `https://github.com/Dendro-X0/KnotTrace/releases/latest`
- `checkForUpdates`: keep last error string for a subtle banner/toast line (optional minimal: set `updateError` state shown near banner area or reuse check path)
- Banner: guard display with `latest_version ?? "unknown"` so `undefined` never renders literally (defense in depth)

### Install path

No change to `install_update` flow once `can_install_in_app` is read correctly. Errors from install should surface (existing throw — ensure UI catches if not already).

## 2. GitHub access diagnosis (`network-core`)

### Trigger

Emit a `BottleneckHint` when **any** of these fail in the current report:

- `github.com` or `api.github.com` in `proxy_path_report` comparisons (proxy and/or direct)
- same hosts in `site_reachability.results`

Fire **even if** `site_access_degraded` is false (single-domain GitHub failure is enough).

### Copy rules (honest)

| Pattern | Title / message direction |
|---|---|
| Proxy-only failure | Active proxy/VPN HTTP path is blocking GitHub; git push / API over 443 will fail; try another node or temporary direct |
| VPN tag + failure (no proxy or both fail) | Tunnel path cannot reach GitHub HTTPS; not an ISP “speed” issue — path/filter |
| Direct+proxy both fail | GitHub unreachable on current network path |

Category: prefer `ProxyPath` when proxy enabled and proxy failed; else `VpnTunnel` when VPN tagged; else `SiteAccess`.

### Reachability targets

When `proxy.enabled` **or** environment has `Vpn` tag, always prepend sensitive hosts (`PROXY_SENSITIVE_DOMAINS`) into reachability targets (same as today’s proxy prepend). Keeps `api.github.com` in the sample under VPN-only.

### Updater diagnosis (optional light)

Do **not** fold updater check failures into HealthReport in this slice (separate lifecycle). Frontend update error string is enough for “could not reach GitHub for updates.”

## 3. Proof

| Layer | Command / check |
|---|---|
| L1 | `cargo test -p network-core --lib diagnosis::` / reachability target test; `cargo test -p network-desktop --lib` if updater tests exist; `pnpm exec tsc --noEmit` in `apps/desktop` |
| L2 | Inspect serialized `UpdateCheck` in unit test |
| L3 | Manual banner + GitHub-block scenario (maintainer) |

## 4. Docs

- `CHANGELOG.md` `[Unreleased]`
- One line in `docs/updater-signing.md` §3: IPC uses snake_case; fallback opens GitHub releases if in-app install unavailable
