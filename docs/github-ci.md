# GitHub CI and releases

## Workflows

| Workflow | File | Triggers |
|----------|------|----------|
| **CI** | `.github/workflows/ci.yml` | Push/PR to `main` or `master` |
| **Release** | `.github/workflows/release.yml` | Push tag `v*`; manual dispatch |
| **Release Android** | `.github/workflows/release-android.yml` | Manual dispatch only |

### CI

Runs on every push and pull request (Linux runners for speed):

- `cargo test -p network-core`
- `cargo check -p network-desktop` (Linux WebKit deps; catches Rust/Tauri compile breaks)
- `npm run build` in `apps/desktop`

Duplicate runs on the same branch are cancelled automatically.

**Docs-only pushes** (`docs/`, `specs/`, `*.md`) skip CI to avoid noise. Release workflows are not part of CI commit checks on `main`.

Rust uses `CARGO_BUILD_JOBS=2` on GitHub runners. For local Windows builds that OOM, use `CARGO_BUILD_JOBS=1`.

### CI vs Release on commit status

| Workflow | When it runs | What you see on `main` pushes |
|----------|----------------|-------------------------------|
| **CI** (3 jobs) | Every push/PR to `main` | Frontend + Rust checks |
| **Release** | Tag push `v*` or manual dispatch | Only when you push a tag on that commit |

If a commit shows **1 failing** check with **Release / Build Windows**, that is usually an old tag-triggered release (for example before `TAURI_SIGNING_PRIVATE_KEY` was set). GitHub keeps failed checks on the commit; a later successful manual release does not remove them.

**v1.1.1** published successfully via Release run #7. The earlier Release #6 failure on the same commit is historical and safe to ignore.

Future `main` pushes without a tag show **CI only** (3 green checks).

### Release

Builds Tauri bundles on **Windows, Linux, macOS, Android, and iOS** and publishes assets to the same GitHub Release tag.

- **Windows**: MSI + NSIS installer
- **Linux**: AppImage + `.deb` + `.rpm`
- **macOS**: `.app.tar.gz` + `.dmg`
- **Android**: signed APK (requires `ANDROID_KEY_*` secrets)
- **iOS**: signed IPA (requires `IOS_*` secrets)
- `latest.json` + `.sig` files for in-app signed updates (generated on the Windows release job)

Uses [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) with `includeUpdaterJson: true`.

**Timing:** the first multi-platform release after a dependency bump can take **20–50 minutes** (cold Rust cache and three runners). Later releases are typically faster with `Swatinem/rust-cache`.

### Required secret (signed updates)

| Secret | Description |
|--------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `apps/desktop/src-tauri/.updater/knottrace.key` |

Generate keys: `./scripts/generate-updater-keys.sh` — see [updater-signing.md](updater-signing.md).

### Mobile releases (signed, required)

Android and iOS are part of the default release path from v1.4.2. **Unsigned mobile builds are not published.** If mobile signing secrets are missing, those jobs are **skipped** (not failed) so desktop installers still publish.

See [mobile-signing.md](mobile-signing.md) for keystore/certificate setup. After adding secrets, re-run **Release** for the tag via **Actions → Release → Run workflow**.

| Platform | Secrets |
|----------|---------|
| Android | `ANDROID_KEY_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS` |
| iOS | `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD`, `IOS_MOBILE_PROVISION` |

The legacy **Release Android** workflow remains for manual rebuilds only.

## First-time setup after push

1. Merge or push `.github/workflows/` to your default branch.
2. Open **Actions** on GitHub and confirm **CI** runs green.
3. Publish release artifacts (see below).

## Tag already pushed without assets

If you tagged `v1.4.0` before adding the release workflow:

1. Push the workflow files to `main`.
2. Go to **Actions → Release → Run workflow**.
3. Enter tag `v1.4.0` and run.

This checks out the tag, builds the installer, and attaches files to the GitHub Release.

Alternatively, delete and recreate the tag after the workflow exists:

```bash
git tag -d v1.4.0
git push origin :refs/tags/v1.4.0
git tag v1.4.0
git push origin v1.4.0
```

## Future releases

1. Bump version in `Cargo.toml`, `package.json`, `tauri.conf.json`.
2. Update `CHANGELOG.md`.
3. Commit, push, tag, and push the tag:

```bash
git commit -am "Release v1.4.1"
git push
git tag v1.4.1
git push origin v1.4.1
```

The release workflow runs automatically on tag push.

If a tag push does not start a run (for example after delete/recreate on the **same commit**), use **Actions → Release → Run workflow** and enter the tag (e.g. `v1.1.1`).

Or move the tag to a new commit and force-push:

```bash
git commit --allow-empty -m "chore: trigger release build"
git tag -f v1.1.1
git push origin main
git push -f origin v1.1.1
```

## Permissions

For private repositories, enable **Actions → General → Workflow permissions → Read and write** so releases can be created.
