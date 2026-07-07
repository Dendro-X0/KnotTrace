#!/usr/bin/env bash
# Regenerate package-lock.json so npm ci passes on Linux CI (Tailwind wasm optional deps).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/apps/desktop"
rm -rf node_modules package-lock.json
npm install
npm ci
echo "package-lock.json is valid for npm ci"
