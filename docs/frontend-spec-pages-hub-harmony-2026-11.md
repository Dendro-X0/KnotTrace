# Frontend spec — Cross-page hub harmony

Status: **shipped in tree**  
Builds on: [ux-pass-pages](./frontend-spec-ux-pass-pages-2026-11.md) · [overview-trust](./frontend-spec-overview-trust-2026-11.md) · [hub-polish](./frontend-spec-hub-polish-2026-11.md)

## Goal

Propagate Overview’s **trust + hub chrome** to DNS, Connect, Protect, and Network Quick so every job page feels like the same product — not a stack of unrelated diagnostic cards.

## Shared rules

| Do | Don’t |
|----|--------|
| One primary `hub-cta` per page state | Two equal teal buttons |
| Uppercase muted section labels / normal-case human badges | Shouty ALL-CAPS status chips |
| Quick: plain summary first, evidence second | Lead with Kernel/API/probe FactLists |
| Network Quick surfaces path evidence when path needs attention | Link-only Quick after Overview “Open Network” |
| Ellipsis `…` in busy labels | Triple-dot `...` |
| Settings stays on existing Settings-hub spec | Re-layout Settings in this band |

## Page deltas

| Page | Change |
|------|--------|
| **FeaturePage** | Primary footer gets `hub-cta`; badges `normal-case`; tighter embedded header |
| **Network Quick** | Metric strip hero + path teaser when break; demote raw probe list |
| **Protect** | Hero badges normal-case; quieter recent-activity; preferences as section chrome |
| **DNS / Connect** | Primary apply/switch = hub-cta; unsupported Connect offers Network CTA |
| **EmptyState** | Decorative icon `aria-hidden` |

## Non-goals

- Backend score / probe changes  
- Advanced Network panel avalanche rewrite (anchors already exist)  
- Overview re-layout  
