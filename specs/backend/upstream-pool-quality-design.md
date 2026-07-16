# Upstream pool quality proof (design)

Status: **slices A–C implemented** (reachability + recurrence + UI claim). Slices D–E (throughput / multi-node sample) deferred.  
Owner: `network-core` diagnosis + reachability (+ history store for recurrence).  
Related: [slow-speed-diagnosis-v1.3.x.md](./slow-speed-diagnosis-v1.3.x.md), [../docs/slow-speed-triage.md](../docs/slow-speed-triage.md)

## Problem

Proxy / “机场” users often see:

- advertised high Mbps, measured download near floor
- intermittent failures on GitHub, Gmail, Google properties
- search works while a subdomain or git push fails
- frequent disconnects and unstable exit IPs

KnotTrace today can mark **proxy-only site failure** (`ProxyPathReport.likely_provider_side`) from a **single** parallel probe of a few HTTPS targets. That is useful but **not rigorous proof of “upstream pool quality is poor”**, because:

1. One snapshot cannot prove intermittency.
2. Reachability ≠ throughput (delay green, download dead).
3. One bad node ≠ whole pool; one bad domain ≠ pool-wide collapse.
4. Direct path may also be censored or filtered; comparison needs careful wording.
5. Auto node hopping is harmful (login / repo / bank IP reputation).

Users correctly conclude the app is optional unless it can **prove** the fault layer with evidence they can trust.

## Goal

When the system proxy (or Clash path) is active, KnotTrace should be able to state one of these **claim grades** with named evidence:

| Claim | Meaning | User action implication |
|-------|---------|-------------------------|
| **Local / DNS / captive** | Fault is before the proxy exit | Fix local or complete portal |
| **Active path impaired** | Current proxy route fails or is slow; direct OK | Stay on node or change *once* if user opts in |
| **Upstream pool quality poor** | Failures / caps recur across time and/or across multiple exits in the same subscription context | **Do not** thrash nodes; change provider or reduce reliance |
| **Inconclusive** | Evidence mixed or insufficient | Show what was measured; no strong blame |

“Prove” here means **falsifiable, differential evidence**, not courtroom certainty and not “we fixed your Mbps.”

## Non-goals

- Magically raise Mbps against seller rate limits
- Forced multi-node hopping / IP rotation
- Process sniffing or MITM of user traffic
- Guaranteeing GitHub/Gmail work on a hostile pool
- Replacing Clash Verge / Mihomo as the proxy client

## Evidence model (L1–L4)

Borrow verification language: a claim is only shown at the confidence its evidence layer supports.

### L1 — Differential reachability (exists today, tighten)

**Signals (already partially implemented):**

- Parallel HTTPS probes: **proxy client vs direct client** on the same domain list
- `proxy_only_failure` per domain
- Error taxonomy: timeout, reset, TLS, DNS, HTTP blocked, proxy

**Strengthen:**

- Separate **probe sets**:
  - **General**: e.g. cloudflare, microsoft, example
  - **Dev / identity sensitive**: `github.com`, `api.github.com`, `mail.google.com`, `accounts.google.com`, `www.google.com`
- Record **error_kind asymmetry** (proxy timeout + direct OK → path; both fail → not pool-only)
- Never set High confidence for “pool” from a single domain alone

**Claim allowed:** *Active path impaired* (Medium/High) when ≥2 proxy-only failures and 0 direct failures on the same snapshot.

### L2 — Throughput differential (missing)

**Signals:**

- On-demand (manual) small download sample **via proxy** and **direct** (or skip direct if policy/censorship makes it meaningless — still record that)
- Compare ratio and absolute floors:
  - proxy download ≪ direct download (when direct measurable)
  - or proxy download ≪ local link capability with upload asymmetrically healthier (user’s Speedtest pattern)

**Claim allowed:** *Active path capacity poor* (Medium) from one run; contributes to pool claim only with L3.

**Caution:** Speedtest domains may be specially shaped; prefer neutral HTTPS sample URLs under app control, documented in settings.

### L3 — Recurrence / intermittency (missing; required for “pool”)

**Signals from history.db:**

- Same `slowdown_shape` / proxy-only failure domains recurring across N checks in a time window
- Flip-flop: domain OK → fail → OK on proxy while direct stays OK
- Correlate with `proxy.enabled` and (when available) Clash **selected node name** + **egress IP** (from existing egress probes)

**Minimum for High “upstream pool quality poor”:**

- Proxy enabled throughout the window
- ≥3 independent checks spanning ≥15–30 minutes (configurable)
- In ≥2 of those checks: ≥2 proxy-only failures **or** proxy throughput floor hit
- Direct path not showing the same failure pattern when measurable
- Preferably ≥2 distinct exit IPs or ≥2 selected nodes in the same subscription show the pattern (proves not “one unlucky node”)

If only one node was ever selected: claim **Active path impaired (recurring)** at High, and **Upstream pool suspected** at Medium — wording must not overclaim “whole pool.”

### L4 — Multi-node sample without thrashing (optional, opt-in)

**Signals:**

- User-triggered “pool sample”: probe reachability (and optional tiny throughput) on a **bounded** set of nodes (e.g. 3–5), with cooldown, **no background rotation**
- Store results keyed by node name; never use this for silent auto-switch

**Claim allowed:** *Upstream pool quality poor* (High) when a clear majority of sampled nodes fail the same sensitive targets or hit the same throughput floor, while direct remains healthier.

## Intermittent partial-site failures (GitHub / Gmail / Google)

### Symptom shape

Extend / specialize `SlowdownShape::PartialSiteFailure` with an explicit provider-facing subtype in diagnosis (name TBD), e.g.:

- `PartialSiteFailure` (generic)
- diagnosis hint category stays `ProxyPath` when differential proves it

### Probe design for intermittency

Single HEAD to `/` is weak for:

- `api.github.com` vs `github.com`
- Gmail / Google account hosts
- git over HTTPS (needs at least a meaningful TLS+HTTP success, not just ICMP/delay)

**v1 probes (safe, no auth):**

| Target | Why |
|--------|-----|
| `github.com` | Web UI |
| `api.github.com` | API / many git HTTPS paths |
| `www.google.com` | General Google |
| `accounts.google.com` or `mail.google.com` | Identity / mail surface |
| Existing general domains | Baseline |

For each: success, latency, `error_kind`, proxy vs direct.

**Git push specifically:** KnotTrace cannot safely perform authenticated git operations. Proof language must be:

> “Proxy path fails TLS/HTTP to GitHub API hosts that git uses; push/pull may fail even if a search page works.”

Optional later: detect local `git` remote failures only if user pastes log or runs an explicit opt-in check — out of v1.

### Intermittency detector

From recent reports:

```text
intermittent(domain) :=
  proxy_enabled
  AND count(proxy success on domain) >= 1
  AND count(proxy failure on domain) >= 1
  AND count(direct failure on domain) == 0   // when direct samples exist
  within last K reports
```

UI copy: “Intermittent on proxy only — classic low-quality or overloaded upstream filtering, not a total local outage.”

## Claim language (strict)

### Allowed (High)

- “These sites failed on the proxy path but succeeded on direct in the same check.”
- “This pattern repeated across multiple checks / exit IPs; upstream quality is the bottleneck.”
- “Throughput on the proxy path is far below direct; seller rate limit or congestion is likely.”

### Allowed (Medium)

- “Active node looks impaired; pool-wide quality not yet proven (only one exit sampled).”
- “Sensitive hosts flip between OK and fail on proxy — upstream instability suspected.”

### Forbidden

- “We improved your speed”
- “Switch nodes to fix this” as default advice when L3/L4 already show pool-wide failure
- Blaming local NIC/DNS when L1 shows proxy-only failures
- Claiming pool quality from Clash delay (ms) alone

## Product posture

1. **Proof over cheerleading** — primary value for proxy users.
2. **No thrash** — never auto-rotate exits to “hunt speed”; document IP reputation risk.
3. **One clear verdict** — Overview shows claim grade + evidence bullets + what was *not* proven.
4. **Action ladder:**
   - Local/DNS → existing assists
   - Active path only → optional single Connect Assist suggestion
   - Pool poor → “change provider or stop sending critical traffic through this pool”; split-tunnel guidance

## Implementation slices (when coding starts)

| Slice | Scope | Proof |
|-------|--------|-------|
| A | Expand verification domain sets + stricter confidence rules in `reachability.rs` / `diagnosis.rs` | Unit tests on report builders |
| B | Persist proxy_path summary fields into trend points / recurrence | Store + diagnosis unit tests |
| C | Intermittency aggregation API + Overview copy | `cargo test` + manual Clash session |
| D | Opt-in dual throughput sample (proxy vs direct) | Manual Network page run |
| E | Opt-in bounded multi-node pool sample (Clash API) | Manual; no background |

Do **not** start Slice E until A–C wording and confidence rules are solid.

## Acceptance criteria

1. With proxy off: no upstream-pool claim.
2. With proxy on, github+google fail proxy-only, direct OK, first time → **Active path impaired**, not full pool claim.
3. Same pattern across ≥3 checks / ≥2 egress IPs → **Upstream pool quality poor** at High.
4. Intermittent github proxy-only flips → Intermediate diagnosis with Medium+ and history evidence listed.
5. UI never recommends rapid node switching when pool claim is High.
6. Existing observe-first / no forced switch policies preserved.

## Open questions

1. On networks where **direct** cannot reach Google/GitHub (censorship), can we still prove pool quality using **relative** multi-node samples only? (Likely yes for L4; L1 wording must change.)
2. Should “pool” be tied to Clash **subscription URL hash** vs merely “proxy enabled”?
3. Minimum sample sizes for throughput without burning user quota on metered or 5× nodes.

## Relationship to “app feels optional”

This feature does **not** make a bad airport fast. It makes KnotTrace **non-optional for blame assignment**: users get evidence to stop debugging their PC, stop thrashing nodes, and decide to change provider or traffic policy. That is the honest product value for this audience.
