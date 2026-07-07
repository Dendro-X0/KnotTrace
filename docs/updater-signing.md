# Updater signing (OSS self-sign)

KnotTrace uses **Tauri minisign** for update artifacts on desktop and mobile. This is separate from Windows Authenticode or Apple notarization — it proves updates came from your release pipeline, not from a random download mirror.

## Two signing layers

| Layer | Purpose | OSS approach |
|-------|---------|----------------|
| **Update signatures** (minisign) | Verify `latest.json` bundles before install | Public key in `tauri.conf.json`; private key in GitHub Secrets |
| **Android APK signing** (JKS) | Android OS trusts the installed APK | Self-generated upload keystore; secrets in CI only |
| **iOS** | Device trusts the app binary | Self-signed/ad-hoc for dev; App Store for production |

## 1. Generate update signing keys (once per project)

```bash
./scripts/generate-updater-keys.sh
```

This creates:

- `apps/desktop/src-tauri/.updater/knottrace.key` — **private** (never commit)
- `apps/desktop/src-tauri/.updater/knottrace.key.pub` — public (committed; also embedded in `tauri.conf.json`)

### GitHub Actions secrets

| Secret | Value |
|--------|--------|
| `TAURI_SIGNING_PRIVATE_KEY` | Full contents of `knottrace.key` **or** path on runner |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Empty string if no password |

If you lose the private key, existing installs cannot receive signed updates — generate a new pair and ship a manual reinstall once.

## 2. Release artifacts

The release workflow sets `includeUpdaterJson: true`. Each GitHub Release publishes:

- `latest.json` — manifest consumed by the in-app updater
- `*.sig` — signatures beside MSI/NSIS/AppImage/APK bundles

Endpoint (configured in `tauri.conf.json`):

```
https://github.com/Dendro-X0/KnotTrace/releases/latest/download/latest.json
```

## 3. In-app behavior

1. On startup, KnotTrace checks the signed endpoint.
2. If a newer version exists, the banner offers **Install update**.
3. Download → verify signature → install → restart (passive mode on Windows).

If signing is misconfigured, the app falls back to opening the GitHub release page.

## 4. Android (mobile)

Prerequisites:

```bash
cd apps/desktop
npm run tauri android init   # once, generates src-tauri/gen/android
```

Generate an upload keystore:

```bash
./scripts/generate-android-keystore.sh
```

CI secrets:

| Secret | Description |
|--------|-------------|
| `ANDROID_KEY_BASE64` | `base64` of your `.jks` file |
| `ANDROID_KEY_PASSWORD` | Keystore password |
| `ANDROID_KEY_ALIAS` | `knottrace` |

The release workflow writes `keystore.properties` before building when `ANDROID_KEY_BASE64` is set.

**Important:** Use the **same** upload key for every release. Changing keys blocks in-place APK updates on user devices.

## 5. iOS (mobile)

- Tauri updater supports iOS for sideloaded/enterprise builds signed with your team certificate.
- App Store distribution uses Apple’s update channel; the in-app checker still surfaces version info and opens the store URL when configured.

## 6. Local signed build

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat apps/desktop/src-tauri/.updater/knottrace.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
cd apps/desktop
npm run tauri build
```

Signed bundles appear under `target/release/bundle/` with `.sig` files.

## Rotating keys

1. Generate a new pair with `./scripts/generate-updater-keys.sh`
2. Update `plugins.updater.pubkey` in `tauri.conf.json`
3. Update `TAURI_SIGNING_PRIVATE_KEY` in GitHub
4. Ship a one-time manual release note asking users to reinstall if auto-update fails
