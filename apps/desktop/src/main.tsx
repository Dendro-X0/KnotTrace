import ReactDOM from "react-dom/client";

import { App } from "./App";
import { LocaleProvider } from "./hooks/useLocale";
import { UiDensityProvider } from "./hooks/useUiDensity";
import { initLocale } from "./lib/i18n";
import { initTheme } from "./lib/theme";
import "./styles.css";

initTheme();
initLocale();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <LocaleProvider>
    <UiDensityProvider>
      <App />
    </UiDensityProvider>
  </LocaleProvider>,
);
