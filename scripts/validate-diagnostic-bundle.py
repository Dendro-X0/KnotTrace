#!/usr/bin/env python3
"""Share-ops Band S2 + paste-trust Band H1 — triage helper for community-pasted diagnostic JSON.

Usage:
  python scripts/validate-diagnostic-bundle.py path/to/bundle.json
  python scripts/validate-diagnostic-bundle.py -   # read stdin

Exit 0 when schema keys look like knottrace.diagnostic_bundle.v1 and default
redaction posture holds (no hostname field; SSID not opted in unless ALLOW_SSID=1).
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

REQUIRED_TOP = (
    "schema_version",
    "exported_at",
    "app",
    "redaction",
    "os",
    "score",
    "proxy",
    "tor",
    "vpn_detected",
    "probe_table",
    "recommendations",
    "integrity_flags",
    "config_timeline",
    "not_checked",
    "ai_prompt_template",
)
REQUIRED_SCORE = ("score", "summary")
EXPECTED_SCHEMA = "knottrace.diagnostic_bundle.v1"
FORBIDDEN_KEYS = frozenset({"hostname", "host_name", "computer_name", "wifi_ssid", "ssid"})


def load(path: str) -> object:
    if path == "-":
        raw = sys.stdin.read()
    else:
        raw = Path(path).read_text(encoding="utf-8")
    if not raw.strip():
        raise SystemExit("input is empty")
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"JSON unparsable: {exc}") from exc


def walk_keys(node: Any, path: str = "$") -> list[str]:
    hits: list[str] = []
    if isinstance(node, dict):
        for key, value in node.items():
            here = f"{path}.{key}"
            if key.lower() in FORBIDDEN_KEYS:
                hits.append(here)
            hits.extend(walk_keys(value, here))
    elif isinstance(node, list):
        for i, item in enumerate(node):
            hits.extend(walk_keys(item, f"{path}[{i}]"))
    return hits


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] in {"-h", "--help"}:
        print(__doc__.strip(), file=sys.stderr)
        return 2

    data = load(argv[1])
    if not isinstance(data, dict):
        raise SystemExit("bundle root must be a JSON object")

    missing = [k for k in REQUIRED_TOP if k not in data]
    if missing:
        raise SystemExit(f"missing required keys: {missing}")

    schema = data.get("schema_version")
    if schema != EXPECTED_SCHEMA:
        raise SystemExit(f"unexpected schema_version: {schema!r} (want {EXPECTED_SCHEMA})")

    score = data.get("score")
    if not isinstance(score, dict) or any(k not in score for k in REQUIRED_SCORE):
        raise SystemExit("score must include numeric score + summary")

    if not isinstance(data.get("probe_table"), list):
        raise SystemExit("probe_table must be a list")
    if not isinstance(data.get("recommendations"), list):
        raise SystemExit("recommendations must be a list")
    if not isinstance(data.get("integrity_flags"), list):
        raise SystemExit("integrity_flags must be a list")

    app = data.get("app")
    if not isinstance(app, dict) or "version" not in app:
        raise SystemExit("app.version missing (needed for triage)")

    redaction = data.get("redaction")
    if not isinstance(redaction, dict):
        raise SystemExit("redaction object missing")
    include_ssid = bool(redaction.get("include_ssid"))
    allow_ssid = os.environ.get("ALLOW_SSID", "").strip() in {"1", "true", "yes"}
    if include_ssid and not allow_ssid:
        raise SystemExit(
            "redaction.include_ssid is true — refuse for public triage "
            "(set ALLOW_SSID=1 only for private review)"
        )

    leaks = walk_keys(data)
    # link_summary may mention ssid= when include_ssid was true; already gated above.
    if leaks:
        raise SystemExit(
            "forbidden identity keys present (hostname/SSID fields are not for public paste):\n  - "
            + "\n  - ".join(leaks)
        )

    print(
        "bundle OK:",
        f"schema={schema}",
        f"os={data.get('os')}",
        f"app={app.get('version')}",
        f"score={score.get('score')}",
        f"include_ssid={include_ssid}",
        f"integrity_flags={len(data.get('integrity_flags') or [])}",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
