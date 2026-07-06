# DNS Integrity / Poisoning Detection (v0.7) - Design Spec

## Goal

Add a lightweight DNS integrity layer that can detect likely poisoning or resolver tampering scenarios and provide safe, user-consented remediation suggestions.

This v0.7 design extends the current "health + latency + assist" model with answer-integrity checks, while preserving:

- observe-first behavior
- reversible actions
- low background overhead
- no destructive network changes

## Why this is needed

Current behavior can detect DNS failures/latency degradation and sometimes recover by switching resolvers, but it cannot reliably identify cases where DNS answers are wrong but still fast.

Examples currently under-covered:

- poisoned resolver returns incorrect A/AAAA quickly
- captive/intercept resolver injects synthetic answers
- inconsistent domain answers between system DNS and trusted public DNS

## Non-goals (v0.7)

- Full IDS/IPS or packet-level MITM detection
- Router compromise forensic analysis
- Automatic firewall/routing table repair
- Guaranteed attribution (malicious vs misconfiguration)

## User-visible outcomes

1. New "DNS integrity" signal in health reasoning and protect alerts.
2. When integrity issues are detected, app explains:
   - what mismatch occurred
   - confidence (low/medium/high)
   - suggested next actions
3. Optional "safe fallback resolver" recommendation remains user-consented.

## Detection model

### Inputs

- Existing environment snapshot DNS servers
- Existing DNS probe path
- New cross-resolver answer checks for selected domains

### Verification domains

Initial small curated set (static in code):

- `example.com` (control)
- `cloudflare.com`
- `microsoft.com`
- `github.com`

Notes:

- Keep set small to minimize probe overhead.
- Domains can be expanded in a later release with opt-in custom list.

### Resolver groups

- Local/system resolver path (current configured resolver behavior)
- Trusted comparison resolvers:
  - `1.1.1.1`
  - `8.8.8.8`
  - `9.9.9.9`

### Heuristics (v0.7)

For each verification domain:

1. Query A and AAAA via local path.
2. Query A and AAAA via trusted resolvers.
3. Flag mismatch candidate when:
   - local answer set has no overlap with trusted union answer set, OR
   - local response repeatedly fails while trusted resolvers succeed.

Issue confidence:

- **low**: single-domain mismatch
- **medium**: 2+ domain mismatches
- **high**: 2+ mismatches + repeated failures/instability

False-positive controls:

- Require repeat check before high confidence.
- Skip strict mismatch if all paths are failing (likely general outage).

## Data model additions (network-core)

Add to `types.rs`:

- `DnsIntegrityStatus`:
  - `state`: `ok | caution | suspicious`
  - `confidence`: `low | medium | high`
  - `mismatch_count`
  - `checked_domains`
  - `summary`
  - `details: Vec<DnsIntegrityFinding>`

- `DnsIntegrityFinding`:
  - `domain`
  - `local_answers: Vec<String>`
  - `trusted_answers: Vec<String>`
  - `local_error: Option<String>`
  - `trusted_error_count`
  - `reason`

Integrations:

- Include `dns_integrity: Option<DnsIntegrityStatus>` in health report payload (or parallel endpoint if report size concerns appear).

## Core API additions

In `network-core`:

- `async fn evaluate_dns_integrity(env: &EnvironmentSnapshot) -> Result<DnsIntegrityStatus, CoreError>`

In desktop Tauri:

- command: `get_dns_integrity_status`
- optional inclusion in periodic health updates once performance is validated

## Protect integration

When integrity status is `suspicious`:

- add protect alert:
  - title: "DNS integrity risk detected"
  - message: include confidence + mismatch count
  - actions:
    - "Review DNS Assist"
    - "Run another health check"
    - "Review proxy options"

Notification policy:

- notify if protect enabled and confidence >= medium
- apply existing cooldown semantics

## UI integration (desktop)

Dashboard:

- show compact DNS integrity badge near trends/score
- tooltip/popover with mismatch summary

DNS page:

- add integrity section:
  - state badge
  - recent checked domains
  - mismatched domain list
  - recommendation text

Protect page:

- render integrity alert in alert list when present

## Performance budget

- Additional check should complete within ~1-2 seconds typical path.
- Keep domain list small and use timeout per query.
- Run integrity checks:
  - on manual health check
  - at lower cadence in background (not every short poll), e.g. aligned with existing recheck interval

## Safety and reversibility

- No automatic resolver changes purely from integrity flag in v0.7 default.
- If auto-protect DNS is enabled, require:
  - confidence >= medium
  - explicit setting allowing integrity-triggered fallback

## Observability and persistence

Store compact integrity summaries in history DB with each report:

- `dns_integrity_state`
- `dns_integrity_confidence`
- `dns_integrity_mismatch_count`

This enables trend charting of integrity risk in later UI iterations.

## Rollout plan (atomic steps)

1. **Design/spec (this doc)**  
   Finalize types + heuristics + constraints.

2. **Core type + evaluator slice**  
   Implement core structures and evaluator function with unit tests.

3. **Desktop command + monitor wiring**  
   Expose command and include in health update loop (guarded cadence).

4. **Protect alert integration**  
   Emit integrity-driven alerts and notifications.

5. **UI surfaces**  
   Add badges/details in Overview, DNS, Protect pages.

6. **History persistence + trends prep**  
   Save integrity summary fields; optional chart follow-up.

## Verification plan

### L1 (required per slice)

- `cargo check --workspace`
- `cargo test -p network-core`
- `npm run build` (when desktop touched)

### L2

- targeted tests for mismatch heuristics
- regression tests for existing DNS assist recommendation logic

### L3 (manual runtime)

- simulate:
  - normal DNS (expect `ok`)
  - local resolver failure with trusted success (expect `caution/suspicious`)
  - synthetic mismatch fixture (expect mismatch findings)

## Open questions

1. Should verification domain set be fixed or user-configurable in v0.7?
2. Should integrity checks run in every manual check only, or also background cadence by default?
3. For enterprise environments with split DNS, how should we reduce false positives?

## Exit criteria for v0.7 DNS integrity

- Integrity status is computed and exposed.
- Protect can alert on suspicious integrity state.
- UI can show integrity status and reasons.
- Existing observe/assist/connect/protect flows remain stable.
