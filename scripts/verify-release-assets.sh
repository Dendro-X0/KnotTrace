#!/usr/bin/env bash
# Tryout-clarity Band F1 — verify GitHub Release desktop installer assets.
# Usage: bash scripts/verify-release-assets.sh [tag]
# Default tag: latest non-draft release (gh release list).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI required (https://cli.github.com/)" >&2
  exit 1
fi

TAG="${1:-${VERIFY_RELEASE_TAG:-}}"
if [[ -z "$TAG" ]]; then
  TAG="$(gh release list --limit 20 --json tagName,isDraft,isPrerelease \
    --jq '[.[] | select(.isDraft==false and .isPrerelease==false) | .tagName][0]')"
fi

if [[ -z "$TAG" || "$TAG" == "null" ]]; then
  echo "could not resolve a release tag" >&2
  exit 1
fi

# Strip leading v for filename patterns (KnotTrace_1.9.1_...).
VERSION="${TAG#v}"

echo "==> [F1] verifying release assets for $TAG (version $VERSION)"

ASSETS_JSON="$(gh release view "$TAG" --json tagName,assets)"
python -u - "$ASSETS_JSON" "$VERSION" "$TAG" <<'PY'
import json, sys

raw, version, tag = sys.argv[1], sys.argv[2], sys.argv[3]
data = json.loads(raw)
names = {a["name"] for a in data.get("assets") or []}
if not names:
    raise SystemExit(f"no assets on release {tag}")

required = {
    f"KnotTrace_{version}_x64_en-US.msi": "Windows MSI",
    f"KnotTrace_{version}_x64-setup.exe": "Windows NSIS",
    f"KnotTrace_{version}_amd64.AppImage": "Linux AppImage",
    f"KnotTrace_{version}_amd64.deb": "Linux deb",
    f"KnotTrace-{version}-1.x86_64.rpm": "Linux rpm",
    f"KnotTrace_{version}_aarch64.dmg": "macOS dmg",
    "latest.json": "updater manifest",
}

missing = []
for name, label in required.items():
    if name not in names:
        missing.append(f"{label} ({name})")

# macOS updater tarball name historically omits version; accept either form.
mac_tarballs = [
    "KnotTrace_aarch64.app.tar.gz",
    f"KnotTrace_{version}_aarch64.app.tar.gz",
]
if not any(n in names for n in mac_tarballs):
    missing.append("macOS app.tar.gz (KnotTrace_aarch64.app.tar.gz)")

# Band H2 — signed updater companions for desktop installers.
sig_required = [
    f"KnotTrace_{version}_x64_en-US.msi.sig",
    f"KnotTrace_{version}_x64-setup.exe.sig",
    f"KnotTrace_{version}_amd64.AppImage.sig",
    f"KnotTrace_{version}_amd64.deb.sig",
    f"KnotTrace-{version}-1.x86_64.rpm.sig",
]
for sig in sig_required:
    if sig not in names:
        missing.append(f"updater signature ({sig})")

if missing:
    print("present assets:", ", ".join(sorted(names)), file=sys.stderr)
    raise SystemExit("missing required assets:\n  - " + "\n  - ".join(missing))

print(f"OK: {tag} has Windows MSI+NSIS, Linux AppImage+deb+rpm, macOS dmg, latest.json, installer .sig")
print(f"asset count: {len(names)}")
PY

echo "==> [H2] verifying latest.json updater trust for $TAG"
gh release download "$TAG" --pattern latest.json --dir "$ROOT/.tmp/release-verify" --clobber >/dev/null
python -u - "$ROOT/.tmp/release-verify/latest.json" "$VERSION" "$TAG" <<'PY'
import json, sys
from pathlib import Path

path, version, tag = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
data = json.loads(path.read_text(encoding="utf-8"))
manifest_version = str(data.get("version") or "")
if manifest_version != version:
    raise SystemExit(f"latest.json version {manifest_version!r} != release {version!r}")

platforms = data.get("platforms")
if not isinstance(platforms, dict) or not platforms:
    raise SystemExit("latest.json platforms missing")

# Band U3 — multi-OS in-app updater platforms (single-writer latest.json).
needed = {
    "windows-x86_64",
    "windows-x86_64-nsis",
    "linux-x86_64",
    "darwin-aarch64",
}
missing = sorted(needed - set(platforms))
if missing:
    raise SystemExit(f"latest.json missing updater platforms: {missing}")

for key in needed:
    entry = platforms.get(key) or {}
    if not entry.get("url") or not entry.get("signature"):
        raise SystemExit(f"latest.json platform {key} needs url + signature")

print(f"OK: latest.json version={manifest_version} platforms={sorted(platforms)} (multi-OS desktop)")
PY

echo "==> [F1/H2] release asset verify OK"
