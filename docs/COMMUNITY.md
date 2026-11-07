# KnotTrace for community tryouts

**Current release (tagged):** **v1.11.0** — installers after CI · [CHANGELOG](../CHANGELOG.md) · prior: [v1.10.2](https://github.com/Dendro-X0/KnotTrace/releases/tag/v1.10.2)
**Job:** Path truth — tell whether the break is local, DNS, active path, or destination/policy. Not a speed booster.

This page is the short public entry kit (community-readiness Band **C1**). Maintainer internals stay in [START-HERE.md](./START-HERE.md).

## What it is

KnotTrace is a **desktop** network companion (Windows primary; macOS/Linux also ship) that:

- Runs health checks and shows a plain-language score + diagnosis  
- Explains **where the path breaks** (local vs proxy/VPN/Tor hop)  
- Surfaces **integrity indicators** (proxy on, listeners, ARP/cert deltas, opt-in script-host / mining-like heuristics, …) — never a “you are safe” or “malware found” badge  
- Offers **opt-in, reversible** assists (DNS, Connect, MTU, Windows local caps)  
- Exports a **redacted diagnostic JSON** you can paste into an issue or an external AI  

CompanionShell is a **desktop preview** (`?companion=1`) plus mobile packaging path — **not** a device-proven phone product until L3 is recorded in release notes / [platform-support.md](./platform-support.md).

## What it is not

- Not a speed booster / Mbps guarantee  
- Not antivirus, EDR, or malware removal  
- Not a claim that “absence of indicators means unmonitored”  
- Not automatic proxy node hopping (Connect stays manual / Protect opt-in)  
- Not an Obscur dependency or companion app  

OS feature matrix: [platform-support.md](./platform-support.md).

## Install (Windows)

1. Open [Releases](https://github.com/Dendro-X0/KnotTrace/releases).  
2. Download the latest **Windows** installer (`.msi` or NSIS `.exe`) from the tag assets.  
3. Install and launch **KnotTrace**.  
4. If Windows SmartScreen warns on a new publisher, use “More info → Run anyway” only if you trust this release channel.

macOS/Linux: same Releases page — use the `.dmg` / AppImage / `.deb` / `.rpm` that matches your OS when published for that tag.

## First five minutes

1. Click **Run health check** in the header (or press **Ctrl+K**). The UI starts in **Quick** mode (dismissible tip: path truth, not speed boost).  
2. On **Overview**, read the grade/score, ranked **next steps**, Path / Integrity teasers, and the compact **path/exit journal**. Path may add a tunnel expectation; partial-site failures name a plain layer (local / DNS / active path / destination).  
3. Theme, language (en / zh-Hans), Quick/Advanced density, and the background monitor live under **Settings**. zh-Hans labels remaining English probe text so mixed UI is not silent.  
4. Switch to **Advanced** for full Path Truth detail, or open **Network** for link / local repair (Quick) vs section-jumpable Path (start-here + quiet accordion) in Advanced.  
5. Use **DNS** / **Connect** / MTU / local-caps only when you want a reversible change. Without a Mihomo/sing-box API, **Connect** stays diagnosis-only — no dead Switch button.  
6. If something looks wrong, use **Diagnostic export → Copy JSON** (below) before asking for help.  

## Export a diagnostic bundle (for help)

1. Run a health check first (export stays disabled until a report exists).  
2. On **Overview**, find **Diagnostic export** and click **Copy JSON** (hostname and SSID stay out by default).  
3. Paste into a GitHub issue (preferred) or share privately. Do not post secrets, corporate hostnames you care about, or full unredacted logs.

## Feedback (preferred channel)

Open a GitHub Issue — use the **Bug report** or **Feedback / idea** templates:

https://github.com/Dendro-X0/KnotTrace/issues/new/choose

Include OS, app version, what you expected vs what happened, and optional redacted diagnostic JSON.

## Reliability note

Maintainer device testing may lag. Pre-share gate: `bash scripts/community-gate.sh` (R1 + R4 + paste fixtures). CI also runs headless Path Truth probes on Windows, macOS, and Linux. CompanionShell on phones stays **device-proof pending** — see [platform-support.md](./platform-support.md) / [l3-device-proof.md](./l3-device-proof.md). Community share claims only what the matrix and release notes support. In-app signed updates cover desktop Windows / Linux (AppImage) / macOS via multi-OS `latest.json`.

If you pasted a diagnostic JSON into an issue, maintainers can triage with:

```bash
python scripts/validate-diagnostic-bundle.py path/to/bundle.json
```

The validator refuses hostname/SSID fields so public pastes stay redacted by default.