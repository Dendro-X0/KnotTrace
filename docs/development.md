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

## Project layout

```
network/
├── crates/network-core/     # Engine: probes, diagnosis, assists
├── apps/desktop/            # React UI + Tauri shell
├── specs/backend/           # Design specs
└── docs/                    # Onboarding and release docs
```

## CLI probe (optional)

```bash
cargo run -p network-core --bin network-probe
```

Runs a one-shot health check from the terminal without the desktop UI.
