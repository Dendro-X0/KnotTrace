# Updater signing (OSS self-sign)

KnotTrace uses **Tauri minisign** for update artifacts on desktop and mobile. This is separate from Windows Authenticode or Apple notarization — it proves updates came from your release pipeline, not from a random download mirror.

## Quick setup (first release)

You already have a key pair if `apps/desktop/src-tauri/.updater/knottrace.key` exists. **Do not regenerate** unless you intend to rotate keys and update `tauri.conf.json`.

**Windows (PowerShell):**

```powershell
.\scripts\setup-github-signing-secret.ps1
```

**macOS / Linux / Git Bash:**

```bash
./scripts/setup-github-signing-secret.sh
```

With [GitHub CLI](https://cli.github.com/) logged in (`gh auth login`), either script sets the secret automatically. Otherwise it prints the repo secrets URL and copy instructions.

| Secret | Required? | Value |
|--------|-----------|--------|
| `TAURI_SIGNING_PRIVATE_KEY` | **Yes** | Full file contents of `knottrace.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | **No** | Only if your key file is password-protected. GitHub **cannot** store empty secrets — omit this for default passwordless keys. The workflow treats a missing secret as no password. |

Then: **Actions → Release → Run workflow** → tag `v1.1.1`.

**No key yet?** Generate once:

```bash
./scripts/generate-updater-keys.sh
```

Then run the setup script above. Commit any `knottrace.key.pub` / `tauri.conf.json` pubkey changes if you generated a new pair.

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
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | **Optional** — only when the private key is encrypted |

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

**Important:** Use the **same** upload key for every release. Changing keys blocks in-place APK updates on user devices.

Full install policy: [mobile-signing.md](mobile-signing.md).

## 5. iOS (mobile)

- Requires distribution certificate (`.p12`) and provisioning profile (`.mobileprovision`) in CI secrets.
- Tauri env vars: `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD`, `IOS_MOBILE_PROVISION`.
- Users must trust the developer profile on device before launch.

Full install policy: [mobile-signing.md](mobile-signing.md).

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
