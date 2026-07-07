#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEYSTORE="$ROOT/apps/desktop/src-tauri/.android/knottrace-upload.jks"
mkdir -p "$(dirname "$KEYSTORE")"

if [[ -f "$KEYSTORE" ]]; then
  echo "Keystore already exists at $KEYSTORE"
  exit 0
fi

keytool -genkeypair -v \
  -keystore "$KEYSTORE" \
  -alias knottrace \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -storepass "${ANDROID_KEY_PASSWORD:-knottrace-dev}" \
  -keypass "${ANDROID_KEY_PASSWORD:-knottrace-dev}" \
  -dname "CN=KnotTrace OSS, OU=Mobile, O=KnotTrace, L=Local, S=Local, C=US"

echo ""
echo "Android upload keystore created: $KEYSTORE"
echo "For CI, base64-encode and store as ANDROID_KEY_BASE64:"
echo "  base64 -i \"$KEYSTORE\" | pbcopy   # macOS"
echo "Secrets: ANDROID_KEY_BASE64, ANDROID_KEY_PASSWORD, ANDROID_KEY_ALIAS=knottrace"
