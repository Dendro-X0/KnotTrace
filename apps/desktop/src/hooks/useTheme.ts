import { useEffect, useState } from "react";

import {
  applyThemeMode,
  getStoredThemeMode,
  resolveThemeMode,
  type ThemeMode,
  THEME_STORAGE_KEY,
} from "@/lib/theme";

export function useTheme() {
  const [mode, setMode] = useState<ThemeMode>(() => getStoredThemeMode());
  const resolved = resolveThemeMode(mode);

  useEffect(() => {
    applyThemeMode(mode);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, mode);
    } catch {
      // ignore persistence failures
    }
  }, [mode]);

  useEffect(() => {
    if (mode !== "system") return;

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyThemeMode("system");
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [mode]);

  return { mode, resolved, setMode };
}
