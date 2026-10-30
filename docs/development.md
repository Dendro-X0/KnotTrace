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

From the repository root:

```bash
cargo test -p network-core
cargo check --workspace
cd apps/desktop && npm run build
cd apps/desktop && npm run tauri build
```

| Check | What it proves |
|-------|----------------|
| `cargo test -p network-core` | Core logic unit tests |
| `cargo check --workspace` | Rust workspace compiles |
| `npm run build` | TypeScript + Vite production bundle |
| `npm run tauri build` | Full desktop installer/bundle |

### L3 runtime smoke (no GUI)

```bash
cargo run -p network-core --bin network-probe --quiet > report.json
cargo run -p network-core --bin network-probe --quiet -- --explain > explain.json
cd apps/desktop && npx tsc --noEmit
```

Confirm `report.json` has `path_break` + `integrity.disclaimer`, and `explain.json` has `schema_version: knottrace.local_explain.v1`.

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

## CLI probe (optional)

```bash
cargo run -p network-core --bin network-probe
```

Runs a one-shot health check from the terminal without the desktop UI.
