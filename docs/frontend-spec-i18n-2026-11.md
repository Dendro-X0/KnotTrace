# Frontend spec — Language / i18n (JSON templates)

Status: **shipped in tree**  
Settings home: [frontend-spec-settings-page-2026-11.md](./frontend-spec-settings-page-2026-11.md)

## Goal

Add a **language setting** and a **JSON locale template** system so KnotTrace can grow translations without rewriting UI structure.

## Storage

- Key: `knottrace-locale`
- Values: `en` | `zh-Hans` (extensible)
- Default: `en` (explicit; do not auto-switch OS language until a later band)

## Templates

```text
apps/desktop/src/locales/
  en.json          # canonical / complete template
  zh-Hans.json     # Simplified Chinese (shipped subset; falls back to en)
  _schema.md       # key conventions (optional note in this spec)
```

Nested keys, dot paths in code: `settings.appearance.title`.

Interpolation: `"Checks about every {secs}s"` → `t("…", { secs: 60 })`.

Missing key → fall back to `en`, then the key string.

## Runtime

| Piece | Role |
|-------|------|
| `lib/i18n.ts` | catalog load, `translate`, locale list |
| `hooks/useLocale.ts` | React context + persist + `document.documentElement.lang` |
| Settings → Language | segmented control of available locales |

## Wire-first surfaces (this band)

Nav labels, Settings page, Theme/Density labels, AppHeader page title, Global Search page actions.

Backend probe strings / Protect alert titles remain English until a later copy band.

## Non-goals

- Full app translation in one pass  
- ICU / plural engines  
- Server-fetched locales  
