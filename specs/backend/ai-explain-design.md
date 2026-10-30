# AI explain — opt-in (Path Truth Band 4)

Status: **4A implemented** (local explain) · **4B design only** (cloud opt-in not chartered)  
Product plan: [product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md) §4.1–4.3, §8 order 4  
Depends on: [diagnostic-bundle-design.md](./diagnostic-bundle-design.md) (Band 2), [integrity-board-design.md](./integrity-board-design.md) (Band 3)

```text
HANDOFF ATOMIC STEP: Band 4 design — AI explain (opt-in)
PAUSED / CANCELLED:    none
FORBIDDEN THIS TASK:   cloud send by default; silent elevation; inventing probes
CANONICAL OWNER:       network-core::ai_explain (facts) + apps/desktop AI explain UI
PROOF BEFORE DONE:     design review only this pass
SURFACE:               Tauri desktop
```

## Goal

Help novices understand the latest check in plain language **without** inventing measurements or granting the model admin rights.

Two tiers:

| Tier | Default? | Behavior |
|------|----------|----------|
| **A. On-device explain** | Yes | Deterministic prose from `DiagnosticBundle` / `HealthReport` summaries — no network, no API key |
| **B. Cloud explain** | Off | User opt-in → send **only** redacted bundle + fixed system prompt → show model reply with action chips |

Band 2 already covers “copy JSON for an external AI.” Band 4 adds **in-app** explain and a gated cloud path.

## Non-goals

- Auto-apply DNS / Connect / MTU / caps from model output  
- Streaming model tools that call Tauri commands without confirm  
- Replacing diagnosis / path_break / integrity owners with LLM judgment  
- Bundling a local GGUF/Ollama runtime in v1 (may note as future Tier C)  
- Obscur / shared AI infra  
- Guaranteed Mbps or “you are unmonitored” claims in prompts or UI  

## Product principles (binding)

1. **Facts first** — model (or template) may only rephrase fields present in the bundle.  
2. **Action IDs only** — every suggested next step maps to a stable `action_id` from Band 2, or `manual_out_of_scope`.  
3. **Confirm before mutate** — clicking an action opens the existing Assist surface; apply still requires current confirm + backup flows.  
4. **Opt-in cloud** — Tier B requires an explicit enable + one-time (or per-session) consent that shows the data sheet.  
5. **Off means off** — disabling cloud clears stored API key (if any) preference and blocks send.  
6. **Honesty footer** — reuse integrity disclaimer when integrity indicators are present; never imply surveillance absence.

## User flow

```text
Overview
  ├─ Check (fast/full) — existing
  ├─ Path break / Integrity / Next steps — existing
  ├─ [Explain locally] → Tier A panel (instant)
  └─ [Explain with AI…] → gated
        ├─ if cloud disabled → settings sheet (data sheet + enable)
        └─ if enabled → confirm “Send redacted bundle?” → Tier B result
```

Copy JSON (Band 2) remains available as power-user escape hatch.

## Tier A — On-device explain (ship first inside Band 4)

### Owner

`network-core::ai_explain::explain_locally(bundle | report) -> LocalExplainReport`

Pure function. No I/O.

### Output

```text
LocalExplainReport
├── schema_version: "knottrace.local_explain.v1"
├── headline: string                 # from path_break.title or diagnosis.summary
├── plain_summary: string            # 2–4 sentences, template-assembled
├── likely_cause: string | null      # from diagnosis / path_break hop
├── next_steps: Vec<{ action_id, label, reason }>  # ≤3, from recommendations
├── caveats: Vec<string>             # not_checked + integrity disclaimer if needed
└── source: "local_template"
```

### Template rules

- Prefer `path_break` + `diagnosis` + top recommendation.  
- If `integrity.level != none`, prepend caveat with canonical disclaimer.  
- If `break_hop == none` and score good → “No hop failure classified; see integrity caveats.”  
- Never invent latency numbers not in bundle/report.  
- Language: plain English; no “boost,” “optimize Mbps,” “clean/safe.”

### UI

- Overview: **Explain** button → side panel / expandable card with headline, summary, next-step buttons (`open_dns_assist`, etc. → existing page navigation).  
- No spinner beyond instant render.

### Proof (Tier A)

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p network-core ai_explain` — fixtures for healthy, proxy TLS, Tor SOCKS down, integrity elevated |
| L2 | full `network-core` lib |
| L3 | Overview Explain shows action chips that navigate only |

## Tier B — Cloud explain (opt-in, second slice)

### Settings (`AiExplainSettings`)

Persisted under KnotTrace data dir (same pattern as protect/throughput):

```text
AiExplainSettings
├── cloud_enabled: bool              # default false
├── provider: none | openai_compat   # v1: single OpenAI-compatible HTTPS chat
├── base_url: string                 # default empty; user-set or documented default
├── model: string
├── api_key_present: bool            # never echo raw key to UI logs
├── include_ssid: bool               # default false; passed to bundle builder
├── consent_version: string          # bump when data sheet changes
└── last_consent_at: DateTime | null
```

API key storage: OS keychain / Tauri stronghold-equivalent if already in stack; otherwise encrypted-at-rest file with clear “key leaves this device to provider” warning. **Do not** put key in `HealthReport` or clipboard export.

### Data sheet (must show before first enable and before each send if consent_version changed)

| Field group | Sent? |
|-------------|-------|
| `DiagnosticBundle` curated fields (Band 2 schema) | Yes, if opted in |
| Hostname | No |
| SSID | Only if `include_ssid` |
| Full `HealthReport` / history DB | No |
| API key | To provider only (auth header), not in bundle body |
| Retention | “Provider’s policy — KnotTrace does not retain cloud replies on our servers (there are none). Local reply cache optional, user-clearable.” |

User-visible one-liner:

> KnotTrace will send a redacted diagnostic JSON to the AI provider you configured. Nothing is sent until you confirm. KnotTrace does not operate a KnotTrace-hosted model.

### Request contract

```text
CloudExplainRequest
├── bundle: DiagnosticBundle         # include_ssid from settings
├── system_prompt: fixed string      # = bundle.ai_prompt_template (+ Band 4 hardening addendum)
└── user_prompt: "Explain this KnotTrace bundle for a non-expert. …”
```

**Hardening addendum** (append to system prompt):

- Reply in markdown with sections: Summary, Likely cause, What to do next, What was not checked.  
- Each “What to do next” bullet must end with `(action_id: …)` using only IDs from the bundle.  
- If unsure, say so; do not fabricate probe rows.

### Response handling

```text
CloudExplainResult
├── ok: bool
├── provider: string
├── model: string
├── plain_markdown: string           # raw model text (sanitized)
├── parsed_actions: Vec<action_id>   # extracted / validated against allowlist
├── rejected_actions: Vec<string>    # model proposed unknown IDs
├── error: string | null
└── received_at: DateTime
```

Sanitize: strip HTML; do not execute code fences as commands. Unknown `action_id` → show as text only, not as a button.

### Network

- Desktop-only HTTPS via existing HTTP stack or `reqwest` in a thin `apps/desktop` command (prefer **desktop crate** for secrets; core stays fact-only).  
- Timeout ≤ 60s; cancelable.  
- No background / monitor-path cloud calls.

### IPC

| Command | Role |
|---------|------|
| `get_ai_explain_settings` | Read settings (key redacted) |
| `set_ai_explain_settings` | Update; require consent_version ack when enabling |
| `explain_locally` | Tier A |
| `explain_with_cloud` | Tier B; fails if `!cloud_enabled` or no key |
| `clear_ai_explain_key` | Wipe key |

### Proof (Tier B)

| Layer | Check |
|-------|--------|
| L1 | prompt assembly + action_id parse unit tests (no live network) |
| L2 | settings load/save round-trip |
| L3 | with cloud off, button refuses send; with mock/provider optional manual |
| L4 | n/a unless claiming a specific provider partnership |

## Architecture

```text
HealthReport
    │
    ├─ build_diagnostic_bundle()     (Band 2, existing)
    │
    ├─ explain_locally(bundle)       (Band 4A, network-core)
    │
    └─ [opt-in] desktop explain_with_cloud(bundle, settings)
              └─ HTTPS chat completions (user provider)
```

Do **not** put provider HTTP inside `network-core` if that forces API keys into the core crate. Keep secrets and transport in `apps/desktop/src-tauri`.

## Ship order inside Band 4

| Slice | Deliverable |
|------:|-------------|
| 4A | `LocalExplainReport` + Overview Explain panel + tests |
| 4B | Settings + data sheet + consent + cloud command + action parsing |
| 4C (optional polish) | Local reply cache clear; “copy reply”; provider presets |

Do not start 4B until 4A is green. Do not enable cloud in CI.

## Claim language

Allowed:

- “Here’s a plain-language reading of **this check’s** measurements.”  
- “Suggested in-app next steps (you confirm any change).”  

Forbidden in UI and prompts:

- “You’re safe / not monitored.”  
- “AI fixed your network.”  
- “Guaranteed faster speeds.”

## Open decisions (maintainer)

Record answers before 4B impl:

1. **Provider default:** user-supplied OpenAI-compatible only vs ship a KnotTrace-brokered endpoint (broker = out of scope unless new infra charter).  
2. **Key storage:** OS credential store vs local encrypted file.  
3. **Reply retention:** memory-only vs optional on-disk cache.  
4. **SSID default:** keep false (recommended).

## Acceptance

### Design

- [x] Tier A vs B split documented  
- [x] Data sheet + consent_version called out  
- [x] Action ID allowlist + no silent apply  
- [x] Core vs desktop ownership for secrets  
- [x] Ship order 4A → 4B  

### 4A implementation

- [x] `explain_locally` + `LocalExplainReport` in `network-core`  
- [x] Tauri `explain_locally_report`  
- [x] Overview `LocalExplainPanel` (navigate-only action chips)  
- [x] L1: `cargo test -p network-core --lib ai_explain`  
- [x] L3 (runtime): `network-probe --explain` returns `knottrace.local_explain.v1` with action_ids + disclaimer caveat  
- [ ] L3 (GUI): Overview Explain locally click — blocked here by missing Windows `RC.EXE` for `tauri dev`  

- [ ] Maintainer answers open decisions (before 4B code)  
- [ ] 4B implementation chartered (separate session)  

## Doc links

- [diagnostic-bundle-design.md](./diagnostic-bundle-design.md)  
- [path-break-classifier-design.md](./path-break-classifier-design.md)  
- [integrity-board-design.md](./integrity-board-design.md)  
- [../../docs/product-plan-path-truth-2026-10.md](../../docs/product-plan-path-truth-2026-10.md)  
