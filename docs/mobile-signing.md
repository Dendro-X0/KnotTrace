# Mobile signing and installation

KnotTrace ships **self-signed** Android and iOS builds. Mobile operating systems block unsigned or untrusted binaries — signing is required for installation, not optional polish.

This follows the same OSS posture as desktop updater signing: you control the keys, CI holds the secrets, and users trust your release channel.

## Policy summary

| Platform | Signing layer | Without signing |
|----------|---------------|-----------------|
| **Android APK** | Upload keystore (JKS) | Install blocked or “unknown sources” warning |
| **iOS IPA** | Distribution cert + provisioning profile | Install blocked on device |
| **In-app updates** | Tauri minisign (`latest.json`) | Update banner falls back to GitHub release page |

Unsigned mobile builds are **not** published. Release CI **skips** Android/iOS jobs when signing secrets are missing so desktop releases still succeed.

## Android

### 1. Generate an upload keystore (once)

```bash
./scripts/generate-android-keystore.sh
```

Creates `apps/desktop/src-tauri/.android/knottrace-upload.jks` (gitignored).

### 2. GitHub Actions secrets

| Secret | Value |
|--------|--------|
| `ANDROID_KEY_BASE64` | Base64 of the `.jks` file |
| `ANDROID_KEY_PASSWORD` | Keystore password |
| `ANDROID_KEY_ALIAS` | `knottrace` |

Encode on macOS/Linux:

```bash
base64 -i apps/desktop/src-tauri/.android/knottrace-upload.jks
```

### 3. User installation

1. Download the signed APK from the GitHub Release.
2. On Android, allow installation from your browser/files app if prompted.
3. If the device previously installed KnotTrace signed with a **different** key, uninstall the old build first.

Keep the same upload key for every release. Rotating keys blocks in-place updates.

## iOS

### 1. Apple Developer assets

You need:

- **Distribution certificate** exported as `.p12`
- **Provisioning profile** (`.mobileprovision`) for bundle ID `com.knottrace.app`
- Profile type: **Ad Hoc** or **Enterprise** for direct IPA install; App Store Connect for store distribution

### 2. GitHub Actions secrets

| Secret | Value |
|--------|--------|
| `IOS_CERTIFICATE` | Base64 of the `.p12` certificate |
| `IOS_CERTIFICATE_PASSWORD` | Certificate export password |
| `IOS_MOBILE_PROVISION` | Base64 of the `.mobileprovision` file |

Encode:

```bash
base64 -i knottrace-distribution.p12
base64 -i KnotTrace.mobileprovision
```

Tauri reads these environment variables during `tauri ios build`.

### 3. User installation

1. Download the signed IPA (or install via TestFlight if you use App Store Connect export).
2. On device: **Settings → General → VPN & Device Management** → trust the developer certificate.
3. Open KnotTrace from the home screen.

iOS will not install IPAs that are unsigned or signed with a profile that does not include the device UDID (Ad Hoc).

## Release workflow

Tag pushes run five release jobs:

- Windows, Linux, macOS (desktop)
- **Android** (`build-android`)
- **iOS** (`build-ios`)

See [github-ci.md](github-ci.md) and [updater-signing.md](updater-signing.md) for desktop updater keys.

The legacy **Release Android** workflow remains for manual rebuilds only; prefer the main **Release** workflow from v1.4.2 onward.

## Local mobile builds

```bash
cd apps/desktop
npm run tauri android init   # once
npm run tauri ios init       # once (macOS + Xcode)

# Android (with keystore configured)
npm run tauri android build -- --target aarch64-linux-android

# iOS (with IOS_* env vars set)
npm run tauri ios build -- --target aarch64-apple-ios
```

Set `TAURI_SIGNING_PRIVATE_KEY` for signed updater artifacts alongside platform signing.
