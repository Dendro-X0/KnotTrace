# Product plan — Script risk scan (bounded, not AV)

**Status:** Implemented (S1–S5 in tree)  
**Date:** 2026-11-02  

```text
GOAL:     On-demand section that scans persistence, processes, and high-risk user folders
          for script / mining-like indicators — with honest “indicator only” copy.
NOT:      Full-disk antivirus, signature DB, quarantine/kill, “malware found” / “clean” badges.
POSTURE:  Opt-in · consent · capped roots · user-triggered + optional health-check enrich
```

## Conflict note

Path Truth / COMMUNITY defer **full AV/EDR** forever. A “comprehensive disk scan for any malicious behavior” as marketing language is **out of scope**. This plan ships the closest honest job: a **script risk scan** over bounded locations.

## Job

Protect gains a first-class **Script risk** section: consent, enable-on-check, **Run scan now**, results list. Extends `suspicious_activity` owner — does not invent a second engine.

## Scope

| In | Out |
|----|-----|
| Startup/Run + LaunchAgents persistence | Whole-volume recursive AV |
| Process sample (mining + elevated script hosts) | Signature / cloud reputation |
| Cap-scanned Temp / Downloads / Desktop scripts | Quarantine / remove / disable |
| Integrity indicators + Protect results UI | Claiming absence = safe |

## Bands

| Band | Work |
|------|------|
| S1 | Design spec + frontend note |
| S2 | Disk-root heuristics + `disk_hits` + public `run_script_risk_scan` |
| S3 | Tauri command + companion hook |
| S4 | Protect Script risk section (Quick + Advanced) |
| S5 | Tests + docs stamp |

## Proof

- L1: unit tests for risky script content / roots  
- L2: `cargo test -p network-core suspicious`  
- L3: Protect → consent → Run scan → results or empty honest summary  
