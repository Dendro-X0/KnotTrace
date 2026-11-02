#!/usr/bin/env bash
# Share-ops Band S1 — combined pre-share gate (R1 reliability + R4 headless).
# Prefer this before tagging a community-facing release.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> [S1] community gate: R1 reliability"
bash "$ROOT/scripts/reliability-gate.sh"

echo "==> [S1] community gate: R4 headless regression"
bash "$ROOT/scripts/headless-regression.sh"

echo "==> [S1] community gate OK (R1 + R4)"
