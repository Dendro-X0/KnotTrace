#!/usr/bin/env python3
"""Remove white canvas bleed from KnotTrace icon assets and refresh Tauri sizes."""

from __future__ import annotations

import subprocess
import sys
import shutil
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
DESKTOP = ROOT / "apps" / "desktop"
ICONS = DESKTOP / "src-tauri" / "icons"
MASTER = ICONS / "icon.png"
PUBLIC = DESKTOP / "public" / "knottrace-icon.png"
BG = (3, 17, 45, 255)


def fix_icon(path: Path) -> int:
    image = Image.open(path).convert("RGBA")
    pixels = image.load()
    width, height = image.size
    changed = 0
    for y in range(height):
        for x in range(width):
            red, green, blue, _alpha = pixels[x, y]
            if red > 190 and green > 190 and blue > 190:
                pixels[x, y] = BG
                changed += 1
    image.save(path)
    return changed


def write_sizes() -> None:
    master = Image.open(MASTER).convert("RGBA")
    for name, size in {
        "32x32.png": 32,
        "64x64.png": 64,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
    }.items():
        master.resize((size, size), Image.Resampling.LANCZOS).save(ICONS / name)
    master.resize((128, 128), Image.Resampling.LANCZOS).save(PUBLIC)


def main() -> int:
    if not MASTER.exists():
        print(f"missing master icon: {MASTER}", file=sys.stderr)
        return 1

    for path in (MASTER, PUBLIC):
        if path.exists():
            print(f"fixed {path.name}: {fix_icon(path)} pixels")

    write_sizes()

    # Use npm exec for portability. On Windows, ensure we call an actual executable.
    npm = shutil.which("npm") or ("npm.cmd" if sys.platform.startswith("win") else "npm")
    subprocess.run(
        [npm, "exec", "--", "tauri", "icon", str(MASTER)],
        cwd=DESKTOP,
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
