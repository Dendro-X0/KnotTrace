export type ThemeMode = "light" | "dark" | "system";

export const THEME_STORAGE_KEY = "knottrace-theme";

export function isThemeMode(value: string | null): value is ThemeMode {
  return value === "light" || value === "dark" || value === "system";
}

export function getStoredThemeMode(): ThemeMode {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (isThemeMode(stored)) return stored;
  } catch {
    // localStorage may be unavailable in some WebView contexts
  }
  return "system";
}

export function resolveThemeMode(mode: ThemeMode): "light" | "dark" {
  if (mode === "light" || mode === "dark") return mode;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function applyThemeMode(mode: ThemeMode) {
  const resolved = resolveThemeMode(mode);
  document.documentElement.dataset.theme = resolved;
  document.documentElement.style.colorScheme = resolved;
}

export function initTheme() {
  applyThemeMode(getStoredThemeMode());
}
