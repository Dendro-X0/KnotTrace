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

echo "==> [H3] community gate: diagnostic paste fixtures"
python "$ROOT/scripts/validate-diagnostic-bundle.py" \
  "$ROOT/scripts/fixtures/diagnostic-bundle-ok.json"
if python "$ROOT/scripts/validate-diagnostic-bundle.py" \
  "$ROOT/scripts/fixtures/diagnostic-bundle-hostname-leak.json" >/dev/null 2>&1; then
  echo "expected hostname-leak fixture to fail H1" >&2
  exit 1
fi
echo "H1 leak fixture correctly rejected"

if [[ -n "${VERIFY_RELEASE_TAG:-}" ]]; then
  echo "==> [F4/H2] community gate: verify release assets ($VERIFY_RELEASE_TAG)"
  bash "$ROOT/scripts/verify-release-assets.sh" "$VERIFY_RELEASE_TAG"
fi

echo "==> [S1] community gate OK (R1 + R4 + H3${VERIFY_RELEASE_TAG:+ + F1/H2})"
