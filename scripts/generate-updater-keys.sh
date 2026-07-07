#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEY_DIR="$ROOT/apps/desktop/src-tauri/.updater"
KEY_PATH="$KEY_DIR/knottrace.key"

mkdir -p "$KEY_DIR"

cd "$ROOT/apps/desktop"
CI=true npm run tauri signer generate -- -w "src-tauri/.updater/knottrace.key" -p "" -f

echo ""
echo "Public key (already embedded in tauri.conf.json):"
cat "$KEY_PATH.pub"
echo ""
echo "Add the PRIVATE key to GitHub Actions secrets as TAURI_SIGNING_PRIVATE_KEY:"
echo "  cat $KEY_PATH"
echo ""
echo "Optional password secret: TAURI_SIGNING_PRIVATE_KEY_PASSWORD (leave empty if none)"
