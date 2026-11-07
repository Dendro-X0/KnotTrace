export const MOTION_PREF_STORAGE_KEY = "knottrace-reduce-motion";

export function getStoredReduceMotion(): boolean {
  try {
    return localStorage.getItem(MOTION_PREF_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

export function storeReduceMotion(value: boolean) {
  try {
    if (value) localStorage.setItem(MOTION_PREF_STORAGE_KEY, "1");
    else localStorage.removeItem(MOTION_PREF_STORAGE_KEY);
  } catch {
    // ignore
  }
}

export function applyReduceMotion(value: boolean) {
  if (value) {
    document.documentElement.dataset.reduceMotion = "true";
  } else {
    delete document.documentElement.dataset.reduceMotion;
  }
}

export function initReduceMotion() {
  applyReduceMotion(getStoredReduceMotion());
}
