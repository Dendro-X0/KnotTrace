# Release and installer build

## Build installers (desktop)

```bash
cd apps/desktop
npm install
npm run tauri build
```

Artifacts land under the **workspace** `target/` directory (not `apps/desktop/src-tauri/target/`):

```
target/release/
├── network-desktop.exe
└── bundle/
    ├── msi/             # Windows
    ├── nsis/            # Windows
    ├── appimage/        # Linux
    ├── deb/             # Linux
    ├── rpm/             # Linux
    └── dmg/             # macOS
```

Exact files vary by runner OS.

## Pre-push checklist

- [ ] `cargo test -p network-core` passes
- [ ] `cargo check --workspace` passes
- [ ] `npm run build` passes in `apps/desktop`
- [ ] `npm run tauri build` completes without errors
- [ ] Version aligned: `Cargo.toml`, `package.json`, `tauri.conf.json`, `CHANGELOG.md`
- [ ] `CHANGELOG.md` updated for the release
- [ ] Smoke test installer on each desktop OS target you publish (Windows/macOS/Linux)

## Version bump

Update these together:

| File | Field |
|------|-------|
| `Cargo.toml` | `[workspace.package] version` |
| `apps/desktop/package.json` | `version` |
| `apps/desktop/src-tauri/tauri.conf.json` | `version` |
| `CHANGELOG.md` | New section + date |

## Git push

```bash
git status
git add .
git commit -m "Release v1.5.0"
git push -u origin main
```

Tag when ready:

```bash
git tag v1.5.0
git push origin v1.5.0
```

## GitHub Actions

CI and release workflows live in `.github/workflows/`. See [docs/github-ci.md](github-ci.md) for:

- Running CI on push/PR
- Publishing Windows/macOS/Linux installer assets to a GitHub Release
- Re-building an existing tag (e.g. `v1.4.1`) via **Actions → Release → Run workflow**

## Mobile plan

Android and iOS installers ship on tag pushes from v1.4.2.

- Signing policy and install steps: [mobile-signing.md](mobile-signing.md)
- Legacy manual workflow: `.github/workflows/release-android.yml`

## Notes

- First `tauri build` may take several minutes (Rust release compile + bundling).
- On machines with limited virtual memory, use `CARGO_BUILD_JOBS=1 npm run tauri build` to reduce parallel rustc jobs.
- Ensure `node_modules` and `target/` are not committed (see root `.gitignore`).
- DNS Assist apply requires elevated privileges on some Windows interfaces; observe-only features work without admin.
