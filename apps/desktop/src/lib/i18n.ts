import en from "@/locales/en.json";
import zhHans from "@/locales/zh-Hans.json";

export const LOCALE_STORAGE_KEY = "knottrace-locale";

export type LocaleCode = "en" | "zh-Hans";

export type LocaleCatalog = typeof en;

export interface LocaleOption {
  code: LocaleCode;
  name: string;
  nativeName: string;
}

const CATALOGS: Record<LocaleCode, LocaleCatalog> = {
  en: en as LocaleCatalog,
  "zh-Hans": mergeCatalog(en as LocaleCatalog, zhHans as DeepPartial<LocaleCatalog>),
};

export const LOCALE_OPTIONS: LocaleOption[] = [
  {
    code: "en",
    name: en.meta.name,
    nativeName: en.meta.nativeName,
  },
  {
    code: "zh-Hans",
    name: zhHans.meta.name,
    nativeName: zhHans.meta.nativeName,
  },
];

type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function mergeCatalog<T extends Record<string, unknown>>(base: T, overlay: DeepPartial<T>): T {
  const out: Record<string, unknown> = { ...base };
  for (const [key, value] of Object.entries(overlay)) {
    if (value === undefined) continue;
    const current = out[key];
    if (isObject(current) && isObject(value)) {
      out[key] = mergeCatalog(current, value as DeepPartial<typeof current>);
    } else {
      out[key] = value;
    }
  }
  return out as T;
}

export function isLocaleCode(value: string | null | undefined): value is LocaleCode {
  return value === "en" || value === "zh-Hans";
}

export function getStoredLocale(): LocaleCode {
  try {
    const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (isLocaleCode(stored)) return stored;
  } catch {
    // localStorage may be unavailable
  }
  return "en";
}

export function catalogFor(locale: LocaleCode): LocaleCatalog {
  return CATALOGS[locale] ?? CATALOGS.en;
}

function lookup(catalog: LocaleCatalog, path: string): string | undefined {
  const parts = path.split(".");
  let cursor: unknown = catalog;
  for (const part of parts) {
    if (!isObject(cursor) || !(part in cursor)) return undefined;
    cursor = cursor[part];
  }
  return typeof cursor === "string" ? cursor : undefined;
}

export function interpolate(
  template: string,
  vars?: Record<string, string | number | null | undefined>,
): string {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) => {
    const value = vars[key];
    return value == null ? "" : String(value);
  });
}

/** Translate a dotted key; missing keys fall back to English, then the key. */
export function translate(
  locale: LocaleCode,
  key: string,
  vars?: Record<string, string | number | null | undefined>,
): string {
  const primary = lookup(catalogFor(locale), key);
  const fallback = locale === "en" ? undefined : lookup(CATALOGS.en, key);
  return interpolate(primary ?? fallback ?? key, vars);
}

export function applyDocumentLang(locale: LocaleCode) {
  document.documentElement.lang = locale === "zh-Hans" ? "zh-Hans" : "en";
}

export function initLocale() {
  applyDocumentLang(getStoredLocale());
}
