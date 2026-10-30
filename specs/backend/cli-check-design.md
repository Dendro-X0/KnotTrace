# Headless CLI — `check --json` (Path Truth)

Status: **implemented**  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §7 optional later, NEXT  
Binary: `network-probe` (`crates/network-core`)

## Goal

Give power users and local agents a **stable headless check** that prints JSON on stdout — no Tauri, no GUI.

## Non-goals

- Applying repairs / DNS assist from CLI  
- Cloud AI calls  
- History enrich (desktop-only; same as today’s probe)  
- Replacing the desktop app

## Canonical owner

| Piece | Owner |
|-------|--------|
| Arg parse + output | `network-core::cli_args` + bin `network-probe` |
| Check engine | `run_health_check_with_settings` |

## Interface

```text
network-probe check --json [--fast|--full] [--explain|--bundle]
network-probe check --pretty ...          # human-oriented JSON (indent)
network-probe --help | check --help

# Backward compatible (unchanged):
network-probe                 → pretty HealthReport JSON
network-probe --explain       → pretty local explain JSON
```

| Flag | Meaning |
|------|---------|
| `check` | Subcommand (required for new form) |
| `--json` | Compact JSON (`serde_json::to_string`) |
| `--pretty` | Indented JSON (default when neither `--json` nor `--pretty`) |
| `--fast` / `--full` | `CheckProfile` (default **full**) |
| `--explain` | Emit local explain (`knottrace.local_explain.v1`) instead of full report |
| `--bundle` | Emit diagnostic bundle (`knottrace.diagnostic_bundle.v1`) instead of full report |

`--explain` and `--bundle` are mutually exclusive.

## Exit codes

| Code | When |
|-----:|------|
| 0 | Check succeeded; JSON written to stdout |
| 1 | Probe / environment failure |
| 2 | Bad usage |

Stderr only for errors and `--help`.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core cli_args` |
| L2 | `cargo test -p network-core --lib` |
| L3 | `cargo run -p network-core --bin network-probe --quiet -- check --json` emits `path_break` + `integrity` |

## Acceptance

- [x] `check --json` compact HealthReport  
- [x] `--explain` / `--bundle` schemas  
- [x] Legacy bare invoke still pretty-prints report  
- [x] Docs + product plan NEXT updated  
