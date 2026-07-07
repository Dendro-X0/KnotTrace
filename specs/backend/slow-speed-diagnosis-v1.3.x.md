# Slow-speed diagnosis (v1.3.x)

## Problem

KnotTrace already detects many useful signals:

- DNS latency and integrity
- Site reachability
- Captive portal hints
- Guest/public network context
- Public egress IP consistency
- Bufferbloat-lite
- MTU fragmentation risk
- Proxy/VPN/Tor presence

But the current `diagnosis.rs` still mostly ranks issues as isolated hints instead of a structured slowdown diagnosis model. Users often say only "the internet is slow," while the real problem may be:

- DNS delay before page start
- bufferbloat under load
- proxy/tunnel path failure
- captive portal / guest Wi-Fi restriction
- MTU fragmentation
- local-only contention or adapter issues

## Goal

Evolve `network-core` diagnosis from a flat bottleneck list into a **symptom-aware slowdown triage model** that:

1. classifies the likely slowdown shape
2. ranks the most plausible bottleneck layer
3. expresses confidence
4. keeps auto-actions conservative and reversible

## Principles

1. **Observe first** — diagnosis should not assume every slowdown is bandwidth-related.
2. **Layer before fix** — identify DNS vs path vs tunnel vs local-link vs saturation.
3. **Confidence over certainty** — prefer medium-confidence guidance over fake precision.
4. **Safe automation only** — diagnosis can support assists, but should not justify intrusive actions by itself.
5. **Cross-platform core** — primary logic lives in `network-core`; platform-specific enrichments are additive.

## Current state

`crates/network-core/src/diagnosis.rs` already emits:

- internet unreachable / loss / latency
- gateway instability
- DNS failure / slow
- DNS integrity
- site access
- public network / captive portal
- egress instability
- Tor / proxy / VPN path
- Wi-Fi / cellular
- bufferbloat / MTU fragmentation

This is a strong base, but it has gaps:

- no explicit "symptom shape" layer
- no confidence field
- no distinction between "raw throughput is low" and "interactive latency is bad"
- no concept of "local-only suspicion"
- limited ranking rules when multiple hints coexist

## v1.3.x scope

This phase should improve diagnosis logic and wording without introducing risky new automation.

### Add

- slowdown symptom classification in diagnosis
- confidence-based summary wording
- better priority rules for DNS vs captive vs MTU vs bufferbloat
- a clear separation between:
  - browsing-start problems
  - under-load responsiveness problems
  - path-specific reachability problems
  - public-network restriction problems

### Do not add

- process-level traffic inspection
- forced proxy node switching
- OS-specific tuning changes
- NIC driver or duplex mutation

## Proposed model

### New enum

Add a new diagnosis-facing concept such as:

- `SlowdownShape::PageStart`
- `SlowdownShape::UnderLoadLag`
- `SlowdownShape::PartialSiteFailure`
- `SlowdownShape::RestrictedNetwork`
- `SlowdownShape::TunnelOverhead`
- `SlowdownShape::LinkLocalIssue`
- `SlowdownShape::GeneralDegradation`

This can either:

1. live directly in `types.rs`, or
2. remain internal to `diagnosis.rs` until the UI needs it

### Confidence

Add a confidence level for the primary diagnosis:

- `high`
- `medium`
- `low`

This should reflect evidence quality, not severity.

Examples:

- captive portal redirect detected → high confidence restricted-network diagnosis
- MTU hint plus VPN/Tor/proxy tunnel present → high confidence path-fragmentation diagnosis
- only high latency without corroborating signals → medium or low confidence

## Ranking rules

### 1. Restricted-network wins early

If captive portal is confirmed, or guest/public network context plus DNS/reachability signals strongly suggest restriction:

- prioritize `RestrictedNetwork`
- primary bottleneck should usually be `CaptivePortal` or `PublicNetwork`
- summary should say the path may require sign-in or is unsafe/unrestricted

### 2. Page-start problems outrank generic "slow internet"

If DNS latency is high and either integrity mismatches, repeated lookup failures, or captive behavior are present:

- prefer `PageStart`
- primary bottleneck should usually be `DnsFailure`, `DnsSlow`, or `DnsIntegrity`
- avoid wording that implies low bandwidth

### 3. Under-load lag should outrank generic Wi-Fi/VPN hints

If bufferbloat is moderate/severe:

- prefer `UnderLoadLag`
- do not let generic Wi-Fi info or VPN overhead become the headline diagnosis

### 4. Partial site failure should point to path-specific causes

If site reachability is degraded:

- prefer `PartialSiteFailure`
- if proxy/tunnel active, rank `ProxyPath`, `TorTunnel`, `VpnTunnel`, or `MtuFragmentation` above generic internet latency

### 5. Tunnel overhead should not mask restricted-network or MTU findings

If VPN/Tor is active but a stronger cause exists:

- do not default to "VPN overhead" as the main diagnosis
- keep tunnel overhead as supporting context unless it has unique evidence

### 6. Link-local issues should outrank ISP speculation

If gateway loss/latency is bad:

- prefer local-link interpretation first
- suggest Wi-Fi/cable/router checks before ISP blame

## Suggested summary patterns

Examples of desired summaries:

- "Pages are likely slowing down before they start loading. DNS or captive portal behavior is the strongest signal."
- "Your connection has acceptable raw reachability, but latency spikes badly under load. This looks more like bufferbloat than a bandwidth cap."
- "Some sites are failing only on the current path. A proxy, tunnel, or MTU issue is more likely than a total outage."
- "This looks like a restricted or guest network. Complete sign-in first, then re-check."

Avoid:

- "Internet is slow" as a generic headline
- "Try another node" unless a proxy path is actually implicated
- strong wording without supporting signals

## UI and product impact

The diagnosis model should support:

- better Overview summaries
- smarter Network recommendations ordering
- clearer Protect alerts
- future symptom-specific empty states / tips

This does **not** require a major UI rewrite in v1.3.x. A better `summary`, `primary_bottleneck`, and hint ordering is enough to improve the experience.

## Suggested implementation steps

### Step 1

Refactor `diagnosis.rs` to derive an internal slowdown shape before generating hints.

### Step 2

Use shape-aware ranking rules when sorting hints, instead of severity-only ordering.

### Step 3

Add confidence-aware summary text.

### Step 4

Keep existing hint categories, but improve their priority when:

- DNS signals cluster
- captive/public network signals cluster
- MTU and tunnel signals cluster
- bufferbloat appears under otherwise healthy reachability

## Exit criteria

- Diagnosis summary distinguishes at least:
  - page-start slowdown
  - under-load lag
  - partial site/path failure
  - restricted/public network problems
- Bufferbloat no longer gets buried under generic Wi-Fi path hints
- DNS-driven browsing delays no longer read like generic low-speed problems
- Proxy/VPN/Tor overhead stays contextual unless it is the strongest supported cause
- Existing tests still pass, with added tests for new ranking behavior

## Future follow-ups

- local-contention suspicion model
- comparative direct-vs-proxy timing
- platform-specific evidence:
  - Windows autotuning / RSS / power
  - Linux negotiated link speed / duplex counters
  - macOS service order / Private Relay interactions
