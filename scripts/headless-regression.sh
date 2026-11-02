#!/usr/bin/env bash
# Band R4 — headless Path Truth regression pack (community-readiness charter).
# Runs network-probe check --fast --json (+ optional --bundle) and asserts
# required keys so community share tags have an automated L3a smoke.
set -euo pipefail
export PYTHONUNBUFFERED=1

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT_DIR="$ROOT/.tmp/headless-regression"
mkdir -p "$OUT_DIR"

REPORT_JSON="$OUT_DIR/report.json"
BUNDLE_JSON="$OUT_DIR/bundle.json"

echo "==> [R4] network-probe check --fast --json"
cargo run -p network-core --bin network-probe --quiet -- check --json --fast >"$REPORT_JSON"

echo "==> [R4] validate report JSON keys ($REPORT_JSON)"
python - "$REPORT_JSON" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    raw = f.read()
if not raw.strip():
    raise SystemExit("report JSON is empty")
try:
    data = json.loads(raw)
except json.JSONDecodeError as exc:
    raise SystemExit(f"report JSON unparsable: {exc}") from exc

missing = [k for k in ("timestamp", "score", "probe", "environment", "path_break", "integrity") if k not in data or data.get(k) is None]
if missing:
    raise SystemExit(f"report missing required keys: {missing}")

score = data["score"]
if not isinstance(score, dict) or "score" not in score or "summary" not in score:
    raise SystemExit("report.score must include numeric score + summary")

integrity = data["integrity"]
if not isinstance(integrity, dict):
    raise SystemExit("report.integrity must be an object")
for key in ("disclaimer", "indicator_count", "level", "title", "summary"):
    if key not in integrity:
        raise SystemExit(f"report.integrity missing {key}")
if not str(integrity.get("disclaimer") or "").strip():
    raise SystemExit("report.integrity.disclaimer must be non-empty")

path_break = data["path_break"]
if not isinstance(path_break, dict):
    raise SystemExit("report.path_break must be an object")

print("report OK: timestamp + score + integrity.disclaimer + path_break")
PY

if [[ "${HEADLESS_REGRESSION_BUNDLE:-1}" != "0" ]]; then
  echo "==> [R4] network-probe check --fast --json --bundle"
  cargo run -p network-core --bin network-probe --quiet -- check --json --fast --bundle >"$BUNDLE_JSON"

  echo "==> [R4] validate bundle schema ($BUNDLE_JSON)"
  python - "$BUNDLE_JSON" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    raw = f.read()
if not raw.strip():
    raise SystemExit("bundle JSON is empty")
try:
    data = json.loads(raw)
except json.JSONDecodeError as exc:
    raise SystemExit(f"bundle JSON unparsable: {exc}") from exc

schema = data.get("schema_version")
if schema != "knottrace.diagnostic_bundle.v1":
    raise SystemExit(f"unexpected bundle schema_version: {schema!r}")

score = data.get("score")
if not isinstance(score, dict):
    raise SystemExit("bundle.score missing")

print("bundle OK: knottrace.diagnostic_bundle.v1 + score")
PY
else
  echo "==> [R4] skipping bundle (HEADLESS_REGRESSION_BUNDLE=0)"
fi

echo "==> [R4] headless regression pack OK"
