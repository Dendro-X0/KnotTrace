# Frontend spec — Script risk scan

Backend: [script-risk-scan-design.md](../specs/backend/script-risk-scan-design.md)

## Placement

Protect page, **both Quick and Advanced**, directly under alerts (before automatic-action history).

## Component

`ScriptRiskPanel`

- Neutral card (not amber alarm chrome); badge `Indicators only`
- Short one-line scope copy
- Compact consent + “Also run with health checks”
- Primary CTA: **Allow & scan** / **Scan this PC** (one-click; see interactive spec)
- Results: indicator cards with Reveal / Copy + **What to do next** guidance  
- Follow-on: [frontend-spec-script-risk-interactive-2026-11.md](./frontend-spec-script-risk-interactive-2026-11.md)

## Copy bans

- No “malware found”, “virus”, “clean”, “secure”
- Empty: “No matches in this bounded sample. That does not mean you are unobserved or malware-free.”
