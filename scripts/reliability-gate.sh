#!/usr/bin/env bash
# Band R1 — cheap reliability gate (community-readiness charter).
# Mirrors CI core/desktop/frontend without a full tauri installer build.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> [R1] cargo test -p network-core --lib"
cargo test -p network-core --lib

echo "==> [R1] cargo check -p network-desktop"
cargo check -p network-desktop

echo "==> [R1] apps/desktop TypeScript + Vite build"
cd "$ROOT/apps/desktop"
# Prefer existing node_modules locally (Windows EPERM on npm ci is common).
# Set RELIABILITY_GATE_NPM_CI=1 to force a clean install (CI-like).
if [[ "${RELIABILITY_GATE_NPM_CI:-}" == "1" ]] || [[ ! -d node_modules ]]; then
  if [[ -f package-lock.json ]]; then
    npm ci
  else
    npm install
  fi
fi
npx tsc --noEmit
npm run build

echo "==> [R1] reliability gate OK"
