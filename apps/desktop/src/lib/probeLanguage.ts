import type { LocaleCode } from "./i18n.ts";

/** Locales where chrome is translated but many probe narratives remain English. */
export function needsEnglishProbeLabel(locale: LocaleCode | string | null | undefined): boolean {
  return locale === "zh-Hans";
}

export const ENGLISH_PROBE_LABEL_KEY = "probeLanguage.englishMeasurements";
