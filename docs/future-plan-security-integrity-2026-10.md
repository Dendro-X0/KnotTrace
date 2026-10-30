# Future plan — Security, integrity, and path honesty (KnotTrace)

**Date (UTC):** 2026-10-17  
**Status:** Plan only · not implementing in this pass  
**Audience:** Maintainer iterating KnotTrace later (Obscur work continues separately)

```text
GOAL:     Grow KnotTrace beyond “local assist + honest Tor/VPN overhead”
NOT:      Fake Mbps · claim “you are not monitored” · become full AV/EDR
POSTURE:  Observe-first · indicators with confidence · reversible assists
```

## Why

Current product is strong on **benign local** problems (DNS, link, caps, MTU, bufferbloat) and honest about tunnel overhead. It feels thin on **adversarial / integrity** questions and deep **path break** diagnosis (VPN/Tor/corp environments). Obscur-class apps need a sibling tool that tells the truth about the machine and the path — without false Ready or fake speed gains.

## Four pillars

| Pillar | Job | Example signals | Claim language |
|--------|-----|-----------------|----------------|
| **1. Performance** | Shape of “slow” | DNS RTT, idle vs loaded latency, duplex, bufferbloat | Symptom + confidence — never guaranteed Mbps |
| **2. Connectivity / environments** | Why this path fails | Captive portal, PAC/proxy, split tunnel, Tor SOCKS, VPN iface, MTU blackhole | Direct vs Proxy vs Tor — which hop breaks |
| **3. Security posture** | Local attack surface | Unexpected listeners, autoruns, hosts overrides, system proxy flipped, odd resolvers | “Unusual / elevated risk” — not “clean” |
| **4. Monitoring / integrity** | Interception / tampering indicators | Extra root CAs, DNS vs integrity mismatch, ARP/gateway churn, forced proxy, TLS MITM hints | **Indicators only** — never “unobserved” |

Pillars 1–2 exist in rough form today. **3–4 are the upgrade.**

## Monitoring detection — feasibility

**Ship as indicators:**

- System/HTTP(S) proxy enabled without clear user intent  
- New/unexpected trusted root certificates  
- DNS answers diverge from trusted resolvers (extend integrity)  
- Default gateway / ARP identity churn  
- Captive or intercept behavior on HTTPS probes  
- Tor/VPN “up” but TLS handshake resets (path filter / DPI class)

**Do not productize as guarantees:**

- Covert kernel/firmware taps, ISP passive taps, switch SPANs  
- Green badge = “safe from surveillance”  
- Detecting all malicious scripts (AV/EDR scope)

Canonical copy:

> We found **indicators** consistent with interception or forced routing. Absence of indicators does **not** mean you are unobserved.

## Malicious activity (network-centric, not AV)

- Correlate new processes ↔ new outbound endpoints (opt-in)  
- Baseline quiet PC vs persistent egress spikes  
- Flag sudden proxy/DNS/hosts changes  
- Export forensic bundle (routes, proxies, DNS, listeners, cert-store diffs)

Promise surface + timeline + export — not “we blocked the malware.”

## VPN / Tor / weird networks

Ladder (align with Obscur Tor smoke T0–T4 honesty):

1. Local OK? (link, DNS, captive, caps, MTU)  
2. Which path? (Direct / system proxy / VPN / Tor SOCKS)  
3. Where break? (DNS / TCP / TLS / HTTP)  
4. Environment class (home / guest / corp / hotspot / filtered region)

KnotTrace diagnoses *why Tor won’t Ready*; it does **not** patch apps to fake Ready.

## Suggested ship order (when implementing)

1. **Path break classifier** — Direct / Proxy / VPN / Tor + TLS-stage labels  
2. **Integrity board** — proxy, DNS integrity, cert-store deltas, gateway/ARP  
3. **Egress anomaly timeline** — local-first, opt-in, no background heavy scans  
4. Protect-style alerts for forced proxy / new root CA / DNS hijack  

Defer: full AV, anti-spyware certification, Tor speed guarantees.

## Relation to Obscur

**Independent products.** KnotTrace does **not** depend on Obscur (and vice versa). No shared runtime, packaging, or release coupling.  
Broader product plan (AI diagnostic bundles, novice UX, exposure board, ship order): [product-plan-path-truth-2026-10.md](./product-plan-path-truth-2026-10.md).

## Non-goals (this plan doc)

- Implementing code in this pass  
- Changing Obscur Tor gateway to work around LAN `TLS_ERROR`  
- Claiming Tor Verified from KnotTrace probes  

_Last updated: 2026-10-17T15:05:00Z_
