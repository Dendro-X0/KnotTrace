export type UiDensityMode = "quick" | "advanced";

export const UI_DENSITY_STORAGE_KEY = "knottrace-ui-density";

export function isUiDensityMode(value: string | null): value is UiDensityMode {
  return value === "quick" || value === "advanced";
}

/** Default Quick for new installs (calm first session). */
export function getStoredUiDensityMode(): UiDensityMode {
  try {
    const stored = localStorage.getItem(UI_DENSITY_STORAGE_KEY);
    if (isUiDensityMode(stored)) return stored;
  } catch {
    // localStorage may be unavailable in some WebView contexts
  }
  return "quick";
}

export function storeUiDensityMode(mode: UiDensityMode) {
  try {
    localStorage.setItem(UI_DENSITY_STORAGE_KEY, mode);
  } catch {
    // ignore
  }
}
