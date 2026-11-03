import {
  createContext,
  createElement,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import {
  getStoredUiDensityMode,
  storeUiDensityMode,
  type UiDensityMode,
} from "@/lib/uiDensity";

interface UiDensityContextValue {
  mode: UiDensityMode;
  setMode: (mode: UiDensityMode) => void;
  isQuick: boolean;
  isAdvanced: boolean;
}

const UiDensityContext = createContext<UiDensityContextValue | null>(null);

export function UiDensityProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<UiDensityMode>(() => getStoredUiDensityMode());

  const setMode = useCallback((next: UiDensityMode) => {
    setModeState(next);
    storeUiDensityMode(next);
  }, []);

  const value = useMemo(
    () => ({
      mode,
      setMode,
      isQuick: mode === "quick",
      isAdvanced: mode === "advanced",
    }),
    [mode, setMode],
  );

  return createElement(UiDensityContext.Provider, { value }, children);
}

export function useUiDensity() {
  const ctx = useContext(UiDensityContext);
  if (!ctx) {
    throw new Error("useUiDensity must be used within UiDensityProvider");
  }
  return ctx;
}
