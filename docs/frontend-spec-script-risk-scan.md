# Frontend spec — Script risk scan

Backend: [script-risk-scan-design.md](../specs/backend/script-risk-scan-design.md)

## Placement

Protect page, **both Quick and Advanced**, directly under alerts (before automatic-action history).

## Component

`ScriptRiskPanel`

- Neutral card (not amber alarm chrome); badge `Indicators only`
- Short one-line scope copy
- Compact consent + “Also run with health checks”
- Primary CTA: **Run scan** (consent alone is enough; enables settings if needed)
- Results: match count badge + three hit lists + honest empty copy

## Copy bans

- No “malware found”, “virus”, “clean”, “secure”
- Empty: “No matches in this bounded sample. That does not mean you are unobserved or malware-free.”
