# Product plan — KnotTrace “Path Truth” network utility

**Date (UTC):** 2026-10-28  
**Status:** Path Truth Bands **1–5 + 4A/4B + 3D + CLI shipped**  
**Repo:** KnotTrace (`network`) only  
**Audience:** Maintainer

```text
GOAL:     High-performance, versatile tool to diagnose and resolve network issues
          for experts and non-experts (optional AI explain via structured bundles).
POSTURE:  Observe-first · indicators with confidence · reversible assists (opt-in)
INDEPENDENT: KnotTrace and Obscur are separate products — no code, runtime,
             packaging, or release dependencies between them.
DONE:     Band 1–5 · 4A/4B · 3D/3E · CLI · GUI smoke · config timeline · cert multi-OS · v1.8.0
NEXT:     AI explain 4C polish
```

---

## 1. Independence (binding)

| Rule | Meaning |
|------|---------|
| **No dependency on Obscur** | Do not import Obscur crates, share release pipelines, or require Obscur installed |
| **No dependency from Obscur** | Obscur must not require KnotTrace; no joint “must ship together” gate |
| **No merge charter** | UI merge, IPC bridge, or shared monorepo is **out of scope** for this plan |
| **Claim discipline only** | Both products may use similar honesty language; that is editorial, not coupling |

Older notes that called KnotTrace a “sibling” to Obscur are **historical framing only**. This plan treats KnotTrace as a **standalone** network product.

---

## 2. Product vision

**Working name:** KnotTrace (brand) · “Path Truth” (capability theme).

**Job:** Help the user understand *what kind of problem they have*, *where the path breaks*, and *what safe next step to take*—in plain language—across home, guest, corporate, hotspot, VPN, VPS/proxy-pool, and Tor environments.

**For non-experts:** One-click checks plus an optional **AI explain** path that consumes a **structured, redacted diagnostic bundle**—the AI interprets measured facts; it does not invent probes or gain silent admin rights.

**Non-job:** Fake Mbps “boosts,” “you are not monitored,” full AV/EDR, or silent OS mutation.

---

## 3. Capability pillars

| Pillar | User question | Tool behavior |
|--------|---------------|---------------|
| **1. Stability** | Why does it drop / feel jumpy? | Link flaps, Wi‑Fi signal, bufferbloat-lite, captive portal, DHCP / adapter health |
| **2. Performance honesty** | Why is it slow? | DNS RTT, idle vs loaded latency, duplex / caps / MTU, tunnel overhead; **recommendations**, never guaranteed speed |
| **3. Path environments** | Me, VPN, VPS proxy, or Tor? | Ladder: local OK? → path (Direct / system proxy / VPN / Tor SOCKS) → hop (DNS / TCP / TLS / HTTP) → environment class |
| **4. Exposure / integrity** | Is this machine open or steered? | Listeners, unexpected proxy/DNS/hosts, gateway/ARP churn, extra root CAs, forced routing — **indicators only** |

Pillars 1–3 extend shipped KnotTrace (L1–L6 local-first + diagnosis). Pillar 4 is the primary product upgrade for “external access / extraction / surveillance” concerns, with hard claim limits (see §6).

Related prior plan (subset): [future-plan-security-integrity-2026-10.md](./future-plan-security-integrity-2026-10.md).

---

## 4. Novice UX + AI assist

### 4.1 User flow

1. **Check my connection** (fast) or **Deep check** (full).  
2. Overview shows: symptom shape, confidence, likely cause, 1–3 next steps.  
3. Optional **Explain with AI** → exports / sends only a redacted bundle + fixed prompt template.  
4. Any repair (DNS apply, flush DNS, etc.) requires **explicit user confirm**; AI never holds silent elevation.

### 4.2 Diagnostic bundle (AI / power-user contract)

Include:

- Environment class, OS, interface summary (SSID **opt-in**)  
- Probe table by path: DNS / TCP / TLS / HTTP (Direct vs Proxy vs Tor when present)  
- Scores, diagnosis shape, confidence  
- Integrity / exposure flags (proxy on, DNS mismatch, new root CA, listener summary)  
- Recent local config-change timeline (proxy / DNS / hosts) when available  
- Explicit **“what we did not check”** list  

### 4.3 AI contract

- AI consumes **structured facts from KnotTrace**; it must not invent measurements.  
- Every suggestion maps to a **tool action ID** or “manual / out of scope.”  
- Default: on-device summary. Cloud AI is **opt-in** with a clear data sheet (fields sent, retention, off-switch).

---

## 5. Scenario matrix (target coverage)

| Scenario | Signals | User-facing outcome |
|----------|---------|---------------------|
| Weak / crowded Wi‑Fi | Signal %, channel, retries | Wireless link bottleneck + practical tips |
| Ethernet capped / half-duplex | Link facts | Cable/NIC negotiation limit |
| Bad / hijacked DNS | Local vs trusted resolvers | Integrity flag + DNS Assist offer |
| Bufferbloat | Idle vs loaded latency | Latency under load — not “buy more Mbps” |
| System proxy / PAC | Proxy detect + path compare | Traffic is going through a proxy |
| VPN up but broken | Interface up, fail stage | VPN path fails at DNS/TCP/TLS/… |
| VPS / Mihomo / sing-box pool | Upstream pool proof | Active node vs pool quality — detect-only unless user switches |
| Tor | SOCKS detect + compare | Privacy path ≠ speed; where Tor fails |
| Captive / guest / corp | Portal, split DNS, PAC | Environment class + breaking hop |
| Machine exposure | Listeners, high-risk ports, WAN-facing hints | Risk **indicator**: reachable / unusual services |
| Possible interception | Forced proxy, new CA, DNS diverge, ARP churn | Indicators of redirected / inspected traffic |

---

## 6. Exposure & integrity — claim language (binding)

**Ship as indicators:**

- Unexpected listening services; high-risk ports open  
- System / HTTP(S) proxy enabled without clear user intent  
- DNS answers diverge from integrity check  
- Default gateway / ARP identity churn  
- New trusted roots; HTTPS intercept-like behavior  
- Opt-in process ↔ egress correlation (Band 3E; Protect toggle; Windows snapshot)  

**Canonical copy:**

> We found **indicators** consistent with exposure, forced routing, or interception. **Absence of indicators does not mean you are unobserved.**

**Do not productize as guarantees:**

- Covert kernel/firmware taps, ISP passive taps, switch SPANs  
- Green badge = “safe from surveillance”  
- Full malware removal / AV certification  

---

## 7. Architecture direction (when implementing later)

Keep the existing KnotTrace split; grow capabilities inside it:

| Layer | Role |
|-------|------|
| **`network-core` (Rust)** | Probes, diagnosis, integrity, assists — no UI; high performance; unit-tested |
| **`apps/desktop` (Tauri)** | Overview / Network / Protect / Assist / AI explain |
| **Headless CLI** | `network-probe check --json` for power users and local agents — [cli-check-design.md](../specs/backend/cli-check-design.md) |

**Performance:** Fast vs full check profiles; lightweight background monitor; throughput and long egress timelines **manual / opt-in only**.

Mobile: thin wrap or “export bundle” only after desktop path ladder and integrity board are solid.

---

## 8. Implementation ship order

Chartered. Execute in order; do not skip ahead without maintainer OK.

| Order | Band | Outcome | Status |
|------:|------|---------|--------|
| 1 | **Path-break classifier** | Direct / Proxy / VPN / Tor + hop labels | **Core shipped** — [path-break-classifier-design.md](../specs/backend/path-break-classifier-design.md) |
| 2 | **Novice Overview + bundle export** | AI-ready JSON without requiring cloud | **Core shipped** — [diagnostic-bundle-design.md](../specs/backend/diagnostic-bundle-design.md) |
| 3 | **Integrity / exposure board** | Proxy, DNS integrity, listeners, gateway/ARP, cert deltas | **Shipped** (incl. Band 3D ARP/cert snapshots + history enrich) — [integrity-board-design.md](../specs/backend/integrity-board-design.md) |
| 4 | **AI explain (opt-in)** | Redacted bundle + mapped actions + data sheet | **4A + 4B shipped** — [ai-explain-design.md](../specs/backend/ai-explain-design.md) |
| 5 | **Egress anomaly timeline** | Opt-in, local-first | **Core shipped** — [egress-timeline-design.md](../specs/backend/egress-timeline-design.md) |
| — | **Defer** | Auto node rotation, “speed optimizer,” AV-like claims, Obscur integration | Out of scope |

---

## 9. Non-goals

- Coupling releases, CI, or packages to Obscur  
- Claiming Tor Verified / “unmonitored” / guaranteed Mbps  
- Becoming a general mobile IDE (separate future idea, separate repo)  
- Skipping Band 1 to ship AI/cloud first

---

## 10. Doc index links

- [START-HERE.md](./START-HERE.md) — product onboarding  
- [architecture.md](./architecture.md) — current module map  
- [../specs/backend/path-break-classifier-design.md](../specs/backend/path-break-classifier-design.md) — Band 1 design  
- [../specs/backend/diagnostic-bundle-design.md](../specs/backend/diagnostic-bundle-design.md) — Band 2 design  
- [../specs/backend/integrity-board-design.md](../specs/backend/integrity-board-design.md) — Band 3 design  
- [../specs/backend/ai-explain-design.md](../specs/backend/ai-explain-design.md) — Band 4 design  
- [../specs/backend/egress-timeline-design.md](../specs/backend/egress-timeline-design.md) — Band 5 design  
- [../specs/backend/cli-check-design.md](../specs/backend/cli-check-design.md) — headless `check --json`  
- [../specs/backend/process-egress-design.md](../specs/backend/process-egress-design.md) — Band 3E opt-in process↔egress  
- [future-plan-security-integrity-2026-10.md](./future-plan-security-integrity-2026-10.md) — earlier integrity/path-break sketch  
- [../specs/backend/local-first-improvement-roadmap.md](../specs/backend/local-first-improvement-roadmap.md) — L1–L6 shipped direction  

_Last updated: 2026-10-30T10:40:00Z_
