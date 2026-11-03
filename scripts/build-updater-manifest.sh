#!/usr/bin/env bash
# Band U1 — single-writer multi-OS latest.json from GitHub Release .sig assets.
# Usage:
#   bash scripts/build-updater-manifest.sh <tag>           # write .tmp/updater-manifest/latest.json
#   bash scripts/build-updater-manifest.sh <tag> --upload  # also replace latest.json on the release
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI required" >&2
  exit 1
fi

TAG="${1:-}"
UPLOAD=0
if [[ -z "$TAG" ]]; then
  echo "usage: bash scripts/build-updater-manifest.sh <tag> [--upload]" >&2
  exit 1
fi
shift || true
for arg in "$@"; do
  case "$arg" in
    --upload) UPLOAD=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 1 ;;
  esac
done

VERSION="${TAG#v}"
REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
OUT_DIR="$ROOT/.tmp/updater-manifest"
mkdir -p "$OUT_DIR"
rm -rf "$OUT_DIR"/*
WORKDIR="$OUT_DIR/assets"
mkdir -p "$WORKDIR"

echo "==> [U1] downloading updater signatures for $TAG"
# Prefer exact updater payloads; ignore missing optional names.
gh release download "$TAG" --repo "$REPO" --dir "$WORKDIR" --clobber \
  --pattern "KnotTrace_${VERSION}_x64_en-US.msi.sig" \
  --pattern "KnotTrace_${VERSION}_x64-setup.exe.sig" \
  --pattern "KnotTrace_${VERSION}_amd64.AppImage.sig" \
  --pattern "KnotTrace_aarch64.app.tar.gz.sig" \
  --pattern "KnotTrace_${VERSION}_aarch64.app.tar.gz.sig" \
  2>/dev/null || true

# Some releases may only have a subset of patterns matched above; list what we got.
ls -la "$WORKDIR"

BASE="https://github.com/${REPO}/releases/download/${TAG}"

python -u - "$WORKDIR" "$OUT_DIR/latest.json" "$VERSION" "$BASE" "$REPO" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path

workdir = Path(sys.argv[1])
out_path = Path(sys.argv[2])
version = sys.argv[3]
base = sys.argv[4]
repo = sys.argv[5]

def read_sig(name: str) -> str:
    path = workdir / name
    if not path.is_file():
        raise SystemExit(f"missing signature asset: {name}")
    return path.read_text(encoding="utf-8").replace("\r\n", "\n").strip() + "\n"

def platform(url_name: str, sig_name: str) -> dict:
    return {"url": f"{base}/{url_name}", "signature": read_sig(sig_name)}

msi = f"KnotTrace_{version}_x64_en-US.msi"
nsis = f"KnotTrace_{version}_x64-setup.exe"
appimage = f"KnotTrace_{version}_amd64.AppImage"
mac_candidates = [
    "KnotTrace_aarch64.app.tar.gz",
    f"KnotTrace_{version}_aarch64.app.tar.gz",
]
mac_name = None
for candidate in mac_candidates:
    if (workdir / f"{candidate}.sig").is_file():
        mac_name = candidate
        break
if mac_name is None:
    raise SystemExit(
        "missing macOS updater signature "
        "(expected KnotTrace_aarch64.app.tar.gz.sig or versioned form)"
    )

platforms = {
    "windows-x86_64": platform(msi, f"{msi}.sig"),
    "windows-x86_64-msi": platform(msi, f"{msi}.sig"),
    "windows-x86_64-nsis": platform(nsis, f"{nsis}.sig"),
    "linux-x86_64": platform(appimage, f"{appimage}.sig"),
    "linux-x86_64-appimage": platform(appimage, f"{appimage}.sig"),
    "darwin-aarch64": platform(mac_name, f"{mac_name}.sig"),
}

notes = (
    f"See [CHANGELOG.md](https://github.com/{repo}/blob/main/CHANGELOG.md) for details.\n\n"
    "### In-app updates\n"
    "Desktop platforms (Windows, Linux AppImage, macOS) read `latest.json` from this release.\n"
)

manifest = {
    "version": version,
    "notes": notes,
    "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z",
    "platforms": platforms,
}

out_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(f"wrote {out_path}")
print("platforms:", ", ".join(sorted(platforms)))
PY

if [[ "$UPLOAD" == "1" ]]; then
  echo "==> [U1] uploading latest.json to release $TAG"
  gh release upload "$TAG" "$OUT_DIR/latest.json" --repo "$REPO" --clobber
  echo "uploaded latest.json"
fi

echo "==> [U1] updater manifest OK"
