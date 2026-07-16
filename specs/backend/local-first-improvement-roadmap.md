# Local-first network improvement (product direction)

Status: **roadmap — L1–L6 implemented in code**; further work is polish / L0 positioning if incomplete.  
Context: Proxy/“airport” optimization cannot overcome seller rate limits or pool tiering. KnotTrace should stop competing on that axis and **earn value on the local path**, with honest Tor/encrypted-proxy observation.

Related:

- [../docs/slowdown-factors.md](../docs/slowdown-factors.md)
- [../docs/slow-speed-triage.md](../docs/slow-speed-triage.md)
- [../docs/platform-support.md](../docs/platform-support.md)
- [upstream-pool-quality-design.md](./upstream-pool-quality-design.md)
- [public-network-egress-v1.3.md](./public-network-egress-v1.3.md)

## North star

**When the user’s own machine, OS stack, DNS, Wi‑Fi/Ethernet link, or local tunnel settings are the bottleneck, KnotTrace should detect it, explain it, and apply only safe reversible fixes.**

When the bottleneck is an upstream proxy pool, KnotTrace should **prove that** and refuse fake “optimization.”

```text
Value split
├── Primary: local network health (Win / Linux / macOS)
├── Secondary: honest proxy/Tor path diagnosis (no speed miracles)
└── Explicit non-goal: beating dishonest airport sellers on Mbps
```

## Why this shift

| Audience pain | App can help? | How |
|---------------|---------------|-----|
| Bad DNS, captive portal, guest Wi‑Fi | Yes | Detect + reversible DNS / guidance |
| Bufferbloat, Wi‑Fi vs Ethernet, duplex/link | Partially | Detect + recommend; rarely auto-mutate |
| Windows receive-window / power / offload quirks | Partially | Detect + guided opt-in repair |
| Airport pool tiering / 倍率 / intermittency | No (speed) | Proof only ([upstream-pool-quality-design.md](./upstream-pool-quality-design.md)) |
| Tor feels slow | No (inherent) | Compare paths; set expectations |

Without this split, the app stays “optional”: great engineering, weak outcomes for the user’s real bottleneck.

## Pillar A — Local network (cross-platform)

Reuse the factor catalog in `slowdown-factors.md`. Rank work by **user-visible impact × safe apply surface**.

### A1. Detection depth (all OS via `network-core` + thin platform adapters)

| Factor | Signal | Priority |
|--------|--------|----------|
| DNS latency / integrity | Already strong | Keep; macOS/Linux **apply** parity |
| Captive / guest / public context | Exists | Keep; richer portal recovery copy |
| Bufferbloat-lite | Exists | Promote in Overview when under-load lag |
| MTU / fragmentation risk | Exists (hints) | Stronger copy when tunnel present |
| Interface kind & quality | Partial (Wi‑Fi/cellular tags) | Add RSSI/noise where OS allows; Ethernet negotiated speed |
| Gateway / LAN instability | Exists | Keep |
| Local contention suspicion | Weak | Soft hints (no process spyware in v1) |

### A2. Safe reversible assists (expand carefully)

| Assist | Windows | macOS | Linux | Notes |
|--------|---------|-------|-------|-------|
| DNS recommend + apply/restore | Yes | **Gap** | **Gap** | Highest ROI parity work |
| Flush resolver cache (opt-in) | Future | Future | Future | Document elevation needs |
| Prefer Ethernet guidance | Rec | Rec | Rec | No forced disable of Wi‑Fi |
| MTU suggest (opt-in, restore) | Later | Later | Later | Never silent; tunnel-aware |
| TCP autotuning check (Win) | Later | — | — | Detect disabled autotuning; opt-in restore |
| qdisc / SQM guidance | — | — | Rec only | Do not rewrite router config from app |

**Policy:** observe-first; auto only for trusted DNS-class actions already gated by Protect; anything that changes MTU/TCP/link stays **manual confirm + backup + restore**.

### A3. Platform slices (suggested order)

1. **macOS + Linux DNS Assist apply/restore** — closes the largest “we diagnose but can’t help” hole (`platform-support.md`).
2. **Link facts panel** — negotiated speed, Wi‑Fi vs Ethernet, power-save suspicion (Windows first, then Linux ethtool-class facts where available).
3. **Windows TCP autotuning / NIC power** — detect misconfig that caps high-BDP links; opt-in repair.
4. **MTU assist (opt-in)** — only when fragmentation risk + tunnel/proxy tags align.
5. **Bufferbloat education + router SQM pointers** — recommendation quality, not magic buttons.

### A4. Acceptance for “local-first feels useful”

- On a healthy LAN with bad DNS: one-click (or one-confirm) DNS fix improves browsing start time.
- On Wi‑Fi-vs-Ethernet mismatch: user sees a clear “prefer wired” diagnosis with evidence.
- On bufferbloat: user sees under-load lag named correctly, not “buy faster ISP.”
- Packaged builds on Win/macOS/Linux expose the same diagnosis model; apply surfaces differ only where OS APIs require it.

## Pillar B — Encrypted proxies & Tor (honest, not “speed up”)

Tor and similar overlays **trade latency and throughput for anonymity**. “Optimizing Tor for speed” is usually the wrong product promise. KnotTrace should **characterize** the path and prevent mis-attribution to the local NIC.

### B1. Already present

- Tor SOCKS detection / reachability
- Optional egress compare: system path vs Tor SOCKS IP
- Tunnel overhead as a slowdown shape input

### B2. Improve without controlling Tor Browser / tor daemon

| Feature | Behavior |
|---------|----------|
| **Path compare panel** | Latency, reachability sample, egress IP: Direct vs System proxy vs Tor SOCKS (when each exists) |
| **Expectation copy** | Explicit: Tor will be slower; high ping is normal; do not chase Mbps |
| **Bootstrap / SOCKS health** | If SOCKS down or flaky, say Tor client issue — not “internet dead” |
| **Circuit-ish signals (soft)** | Repeated egress IP churn vs sticky exit (from egress probes over time) — observational only |
| **Site matrix** | Same differential idea as proxy path: which hosts fail only on Tor |

### B3. Explicit non-goals for Tor

- Launching, configuring, or “accelerating” Tor
- Bridging, pluggable-transport management
- Claiming Tor can match direct or VPN throughput
- Auto-switching Tor exits (deanonymization / policy risk)

### B4. Other encryption proxies (VPN / WireGuard / system tunnel)

Treat like Tor for diagnosis posture:

- Detect tunnel present (existing VPN hints)
- Compare reachability and latency vs direct when possible
- MTU/fragmentation guidance when large transfers stall
- Do not take over vendor VPN apps

Clash/Mihomo remains Connect Assist territory; **upstream pool proof** stays secondary honesty ([upstream-pool-quality-design.md](./upstream-pool-quality-design.md)), not a speed engine.

## Pillar C — Keep proxy honesty (bounded)

Do not abandon proxy users; change the contract:

1. Prove **active path** vs **pool** vs **local** ([upstream-pool-quality-design.md](./upstream-pool-quality-design.md)).
2. Never recommend node thrashing when pool claim is High.
3. Prefer split-tunnel / “don’t send git through this pool” guidance over fake boosts.

## What not to build next

- Background saturation tests that thrash Wi‑Fi or burn metered data
- Silent MTU/TCP/registry edits
- “AI optimize my airport” node hoppers
- Corporate VPN / captive bypass
- Feature sprawl (themes, DND, etc.) unless local-first pillars ship first

Silence/control (v1.5) can remain; it does not replace local outcome work.

## Proposed roadmap bands

| Band | Theme | Exit criteria |
|------|--------|----------------|
| **L0** | Positioning | README / START-HERE state local-first + proxy proof; stop implying general “speed improve” for proxies |
| **L1** | DNS apply parity | **Done (code)** — macOS `networksetup` + Linux `nmcli`/`resolvectl` apply/restore with backup; parser unit tests; platform doc |
| **L2** | Link & local facts | **Done (code)** — negotiated speed/duplex + Link panel + LinkLocalIssue diagnosis |
| **L3** | Windows local caps | **Done (code)** — TCP auto-tuning + NIC power detect; opt-in restore; Network Local caps panel |
| **L4** | Tor/VPN compare UX | **Done (code)** — Direct / System proxy / Tor SOCKS compare + expectation copy; Network Tunnel compare panel |
| **L5** | Upstream proof slices | **Done (code)** — domains A + claim grades + history recurrence C; Overview/Network panels; thrash-forbidden Next Steps (slices D–E deferred) |
| **L6** | Opt-in MTU | **Done (code)** — fragmentation + tunnel eligibility; backup/restore MTU assist on Win/macOS/Linux; Network panel |

Preferred coding order: **L0 → L1 → L2 → L4** (value without proxy theater), then L3/L5/L6 as capacity allows.

## Claim language

| Allowed | Forbidden |
|---------|-----------|
| “Local DNS is slow; apply a trusted resolver (reversible).” | “We will make your proxy as fast as your ISP plan.” |
| “Under load, latency explodes — bufferbloat likely.” | “Tor optimized.” |
| “Tor SOCKS works but is ~N× slower than direct; expected.” | “Switched nodes to fix GitHub” as default automation |
| “Failures are proxy-only across multiple checks — upstream.” | Implied certainty from one Clash delay number |

## Proof before calling a band done

- **L1:** platform apply/restore round-trip on at least one real adapter per OS; `cargo test -p network-core`
- **L2:** diagnosis shape changes in fixtures when link facts indicate Ethernet 100M vs 1G, etc.
- **L4:** with Tor SOCKS up, UI shows distinct egress and slower path without offering “speed up Tor”
- **L5:** unit tests for claim grades; manual Clash session for intermittency
- **L6:** eligibility unit tests; manual apply/restore round-trip on one adapter

## Open questions

1. Elevation UX on macOS/Linux for DNS apply — polkit vs osascript vs instruct-only first version?
2. How aggressive should Windows autotuning repair be by default (off vs Protect opt-in)?
3. Should Tor compare require an explicit Network-page toggle to avoid surprising Tor users with extra probes?

## Bottom line

Improve the app by **owning the local stack across OSes**, **telling the truth about Tor and VPNs**, and **proving** when encrypted upstreams are the limit. That is how KnotTrace stops being optional for people whose airport is the real bottleneck—and becomes useful for everyone else.
