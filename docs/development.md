# Development

## Prerequisites

- [Rust](https://rustup.rs/) (2021 edition)
- [Node.js](https://nodejs.org/) 20+
- Windows: WebView2 (usually preinstalled on Windows 10/11)

## Setup

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Verification (run before push)

### R1 cheap reliability gate (default)

Matches CI’s three jobs without a full installer build. Prefer this before every push:

```bash
./scripts/reliability-gate.sh
```

| Step | Command | What it proves |
|------|---------|----------------|
| Core | `cargo test -p network-core --lib` | Unit tests |
| Desktop Rust | `cargo check -p network-desktop` | Tauri crate compiles |
| Frontend | `tsc --noEmit` + `npm run build` | Types + Vite production bundle |

CI (`.github/workflows/ci.yml`) runs the same three surfaces on Linux. Docs-only pushes skip CI by design.

### R4 / P2a headless regression pack

Live Path Truth smoke without the GUI. Needs network access; writes JSON under `.tmp/headless-regression/`. CI runs the same script on Ubuntu + macOS (Band P2a).

```bash
./scripts/headless-regression.sh
```

| Step | Command | What it proves |
|------|---------|----------------|
| Fast check | `network-probe check --json --fast` | JSON parses; `timestamp`, `score`, `integrity.disclaimer`, `path_break` present |
| Bundle (default) | same + `--bundle` | `schema_version: knottrace.diagnostic_bundle.v1` + `score` |

Set `HEADLESS_REGRESSION_BUNDLE=0` to skip the bundle pass. Artifacts stay local (gitignored via `.tmp/`). Physical CompanionShell remains [l3-device-proof.md](./l3-device-proof.md) P2b.

### R3 panic hygiene (startup + check path)

Audit posture for community tryouts:

- Desktop `perform_check` / `run_check` return `Result` and emit `health-check-failed` (no unwrap on the check path).
- `network-probe` exits non-zero on check/serialize failure instead of panicking.
- Path Truth `classify_local_break` no longer unwraps optional `dns_integrity` when building evidence.
- Tauri `Builder::run` keeps a single entrypoint `.expect` (process cannot continue without a runtime).

Do not add silent `.unwrap()` / `.expect()` on startup or health-check paths; surface `Result`/`String` errors to the UI or CLI.

### Full release-oriented checks

```bash
cargo test -p network-core
cargo check --workspace
cd apps/desktop && npm run build
cd apps/desktop && npm run tauri build
```

| Check | What it proves |
|-------|----------------|
| `cargo test -p network-core` | Core logic (lib + bins) |
| `cargo check --workspace` | Entire Rust workspace |
| `npm run build` | TypeScript + Vite production bundle |
| `npm run tauri build` | Full desktop installer/bundle |

### L3 runtime smoke (no GUI)

```bash
cargo run -p network-core --bin network-probe --quiet -- check --json > report.json
cargo run -p network-core --bin network-probe --quiet -- check --json --explain > explain.json
cargo run -p network-core --bin network-probe --quiet -- check --json --bundle > bundle.json
cd apps/desktop && npx tsc --noEmit
```

Confirm `report.json` has `path_break` + `integrity.disclaimer`, `explain.json` has `schema_version: knottrace.local_explain.v1`, and `bundle.json` has `knottrace.diagnostic_bundle.v1`.

### L3 GUI click-smoke (WebView2 CDP)

Windows needs SDK **RC.EXE** on `PATH` (example: `Windows Kits/10/bin/<ver>/x64`).

```bash
export PATH="/c/Program Files (x86)/Windows Kits/10/bin/10.0.26100.0/x64:$PATH"
export WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9223 --remote-allow-origins=*"
cd apps/desktop && npm run tauri dev
# other terminal:
cd apps/desktop && npm run smoke:gui
```

`smoke:gui` connects over CDP, verifies Overview Path Truth panels, runs a health check via Ctrl+K, clicks Explain / Copy JSON, then checks Network panels.

## Project layout

```
network/
├── crates/network-core/     # Engine: probes, diagnosis, assists
├── apps/desktop/            # React UI + Tauri shell
├── specs/backend/           # Design specs
└── docs/                    # Onboarding, release, and triage docs
```

Related reading: [START-HERE.md](./START-HERE.md), [architecture.md](./architecture.md), [platform-support.md](./platform-support.md), [slowdown-factors.md](./slowdown-factors.md), [slow-speed-triage.md](./slow-speed-triage.md), [local-first roadmap](../specs/backend/local-first-improvement-roadmap.md).

## CLI probe (headless)

```bash
cargo run -p network-core --bin network-probe -- check --json
cargo run -p network-core --bin network-probe -- check --json --fast
cargo run -p network-core --bin network-probe -- check --json --explain
cargo run -p network-core --bin network-probe -- check --json --bundle
```

One-shot Path Truth check without the desktop UI. Prefer `check --json` for agents; bare `network-probe` still pretty-prints a full report (legacy).
