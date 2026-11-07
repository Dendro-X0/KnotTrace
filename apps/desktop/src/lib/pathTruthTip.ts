export const PATH_TRUTH_TIP_STORAGE_KEY = "knottrace-path-truth-tip-dismissed";

/** Pure dismiss check — pass stored value for tests. */
export function shouldShowPathTruthTip(stored: string | null | undefined): boolean {
  return stored !== "1";
}

export function readPathTruthTipDismissed(): string | null {
  try {
    return localStorage.getItem(PATH_TRUTH_TIP_STORAGE_KEY);
  } catch {
    return null;
  }
}

export function dismissPathTruthTip(): void {
  try {
    localStorage.setItem(PATH_TRUTH_TIP_STORAGE_KEY, "1");
  } catch {
    // localStorage may be unavailable in some WebView contexts
  }
}
