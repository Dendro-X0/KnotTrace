# GitHub CI and releases

## Workflows

| Workflow | File | Triggers |
|----------|------|----------|
| **CI** | `.github/workflows/ci.yml` | Push/PR to `main` or `master` |
| **Release** | `.github/workflows/release.yml` | Push tag `v*`; manual dispatch |

### CI

Runs on every push and pull request:

- `cargo test -p network-core`
- `cargo check --workspace`
- `npm run build` in `apps/desktop`

### Release

Builds the Windows Tauri bundle and creates a GitHub Release with:

- MSI installer (`Network Companion_<version>_x64_en-US.msi`)
- `network-desktop.exe`

Uses [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) with `GITHUB_TOKEN` (no extra secrets required for public repos).

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
