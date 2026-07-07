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

Rust uses `CARGO_BUILD_JOBS=2` on GitHub runners. For local Windows builds that OOM, use `CARGO_BUILD_JOBS=1`.

### Release

Builds the Windows Tauri bundle and creates a GitHub Release with:

- MSI installer (`KnotTrace_<version>_x64_en-US.msi`)
- `latest.json` + `.sig` files for in-app signed updates

Uses [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) with `includeUpdaterJson: true`.

**Timing:** the first Windows release after a dependency bump can take **20–40 minutes** (cold Rust cache). Later releases on the same runner image are usually **8–15 minutes** thanks to `Swatinem/rust-cache`.

### Required secret (signed updates)

| Secret | Description |
|--------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `apps/desktop/src-tauri/.updater/knottrace.key` |

Generate keys: `./scripts/generate-updater-keys.sh` — see [updater-signing.md](updater-signing.md).

### Android releases (optional, manual)

Mobile is not part of the default release path. When ready:

1. Run **Actions → Release Android → Run workflow** with the tag (e.g. `v1.1.1`).
2. Set secrets `ANDROID_KEY_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`.

This avoids a skipped Android job on every desktop release.

## First-time setup after push

1. Merge or push `.github/workflows/` to your default branch.
2. Open **Actions** on GitHub and confirm **CI** runs green.
3. Publish release artifacts (see below).

## Tag already pushed without assets

If you tagged `v0.9.0` before adding the release workflow:

1. Push the workflow files to `main`.
2. Go to **Actions → Release → Run workflow**.
3. Enter tag `v0.9.0` and run.

This checks out the tag, builds the installer, and attaches files to the GitHub Release.

Alternatively, delete and recreate the tag after the workflow exists:

```bash
git tag -d v0.9.0
git push origin :refs/tags/v0.9.0
git tag v0.9.0
git push origin v0.9.0
```

## Future releases

1. Bump version in `Cargo.toml`, `package.json`, `tauri.conf.json`.
2. Update `CHANGELOG.md`.
3. Commit, push, tag, and push the tag:

```bash
git commit -am "Release v0.9.1"
git push
git tag v0.9.1
git push origin v0.9.1
```

The release workflow runs automatically on tag push.

## Permissions

For private repositories, enable **Actions → General → Workflow permissions → Read and write** so releases can be created.
