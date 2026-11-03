# Frontend spec — Consumer-calm Protect

Status: **shipped in tree**  
Extends: [frontend-spec-ux-pass-pages-2026-11.md](./frontend-spec-ux-pass-pages-2026-11.md)

## Goal

Protect should feel closer to a calm consumer security/performance tool: **one status**, **short plain alerts**, **one primary action per region**, technical detail on demand.

## Structure

1. **Trust hero** — plain headline + one sentence (no “Network looks use caution on”)  
2. **Alert rows** — title in everyday language; body collapsed by default; no per-alert “Run health check” (header already owns that)  
3. **Script risk** — short scope line, consent tucked under controls, **Run scan** as the only primary in that card  
4. **Activity** — Quick: collapsed by default or 1–2 quiet lines  
5. **Preferences** — Quick: behind “Protection preferences” disclosure  

## Copy map (UI display)

| Backend-ish title | Everyday title |
|-------------------|----------------|
| Forced routing / proxy path | Proxy path looks forced |
| High-risk listener detected | Open network service found |
| Script-host persistence… | Startup script pattern found |
| Mining-like process… | Mining-like process pattern |

Never claim clean/safe/virus-free.
