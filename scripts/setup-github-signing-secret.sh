#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEY_PATH="$ROOT/apps/desktop/src-tauri/.updater/knottrace.key"
PUB_PATH="$KEY_PATH.pub"
REPO="${GITHUB_REPOSITORY:-Dendro-X0/KnotTrace}"

if [[ ! -f "$KEY_PATH" ]]; then
  echo "Private key not found at:"
  echo "  $KEY_PATH"
  echo ""
  echo "Generate a new pair first:"
  echo "  ./scripts/generate-updater-keys.sh"
  exit 1
fi

if [[ ! -f "$PUB_PATH" ]]; then
  echo "Missing public key: $PUB_PATH"
  exit 1
fi

echo "KnotTrace updater signing — GitHub secret setup"
echo "=============================================="
echo ""
echo "Public key (must match tauri.conf.json plugins.updater.pubkey):"
cat "$PUB_PATH"
echo ""

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  echo "GitHub CLI detected. Setting secrets on $REPO ..."
  gh secret set TAURI_SIGNING_PRIVATE_KEY --repo "$REPO" < "$KEY_PATH"
  gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo "$REPO" --body ""
  echo ""
  echo "Done. Secrets set:"
  echo "  TAURI_SIGNING_PRIVATE_KEY"
  echo "  TAURI_SIGNING_PRIVATE_KEY_PASSWORD (empty)"
  echo ""
  echo "Re-run the release: Actions → Release → Run workflow → v1.1.1"
  exit 0
fi

echo "Manual setup (GitHub CLI not available or not logged in):"
echo ""
echo "1. Open: https://github.com/$REPO/settings/secrets/actions"
echo "2. New repository secret"
echo "   Name:  TAURI_SIGNING_PRIVATE_KEY"
echo "   Value: entire contents of this file (copy all lines):"
echo "           $KEY_PATH"
echo ""
echo "3. New repository secret"
echo "   Name:  TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
echo "   Value: (leave blank — press space or use empty string)"
echo ""
echo "4. Re-run release: Actions → Release → Run workflow → tag v1.1.1"
echo ""
echo "To print the private key for copy/paste (keep it secret):"
echo "  cat \"$KEY_PATH\""
