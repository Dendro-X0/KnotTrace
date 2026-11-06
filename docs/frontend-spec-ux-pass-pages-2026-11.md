# Frontend spec — UX pass (pages)

Status: **P0 shipped in tree** (X1–X5); P2 deferred  
Depends on: [frontend-spec-ui-density-modes.md](./frontend-spec-ui-density-modes.md)

## Designer verdict (current product)

KnotTrace reads as a capable **network companion**, but several pages still feel like **stacked diagnostic cards** rather than guided jobs. Overview Quick Hero is the right direction. The same clarity needs to propagate through chrome and the job pages (DNS / Connect / Protect / Network).

## Archetypes (target)

| Route | Archetype | Job in one sentence |
|-------|-----------|---------------------|
| Overview | Status hero | “How am I, and what should I do?” |
| DNS | Assist / decision | “Should I change DNS, and to what?” |
| Connect | Assist / decision | “Should I switch a proxy node?” |
| Protect | Form + alerts | “Is this network trustworthy, and what may auto-fix?” |
| Network | Tools / evidence | “What does the path look like, and what can I repair locally?” |

## Findings (priority order)

### P0 — Structure

1. **Global header competes with page heroes**  
   AppHeader always shows Monitor badge, poll interval, last trigger, and “Last checked.” On Overview Quick this fights the Hero.  
   **Fix:** Quick → page title + primary check action + compact monitor switch. Demote poll/trigger copy to Advanced or a single muted line.

2. **Double titles on DNS / Connect / Protect**  
   AppHeader already titles the page; FeaturePage repeats the same CardTitle + icon.  
   **Fix:** FeaturePage `chrome="embedded"` — badge + description + body, no second H1-scale title.

3. **Protect is a settings dump**  
   Alerts, history, three opt-in consent blocks, then ~12 undifferentiated toggles. Violates form archetype (grouped sections, progressive disclosure).  
   **Fix (shipped):** Trust **hero** (headline + summary + alerts inline); Script risk as its own card; compact recent-actions preview; settings card with grouped toggles. Quick: hero + script risk + 2-line action preview + core autos.

### P1 — Clarity

4. **Network Quick lacks a plain summary**  
   Probe FactList is correct data, wrong first impression.  
   **Fix:** One-line link summary above probes (“Gateway 1 ms · Internet reachable · DNS 13 ms”).

5. **Connect leads with Kernel / API facts**  
   Fine for power users; intimidating in Quick.  
   **Fix:** Quick shows recommendation + groups + primary CTA; Kernel/API stay in details (API settings already collapsed).

6. **DNS Assist packs integrity editor under the ranking list**  
   Integrity settings are Advanced-adjacent.  
   **Fix:** Quick = ranking + apply/restore + integrity status panel; settings editor Advanced-only.

### P2 — Polish (absorbed by A+B charter)

7. Sidebar “Network” vs Overview path language still overlaps conceptually.  
8. Advanced Network remains a vertical panel avalanche — needs section anchors or tabs later.  
9. Empty states are honest; tighten CTAs to one primary action each.

Owned by [product-plan-desktop-trust-path-truth-2026-11.md](./product-plan-desktop-trust-path-truth-2026-11.md) (**A1** / **A2**); not a separate UX-pass band.

## Non-goals

- CompanionShell redesign  
- New probes / backend claims  
- Marketing/landing visual system inside the app shell  

## Implementation bands

| Band | Scope |
|------|--------|
| X1 | Spec (this file) |
| X2 | AppHeader Quick quiet chrome |
| X3 | FeaturePage embedded chrome; wire DNS/Connect/Protect |
| X4 | Protect grouped settings + Quick density |
| X5 | Network Quick summary strip; Connect/DNS Quick trimming |

## Copy rules (unchanged)

- No “secure / virus-free / clean machine.”  
- Integrity empty: “No indicators this check.”  
- Prefer score/plain summary over diagnosis jargon in Quick heroes.
