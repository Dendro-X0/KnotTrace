import ReactDOM from "react-dom/client";

import { App } from "./App";
import { UiDensityProvider } from "./hooks/useUiDensity";
import { initTheme } from "./lib/theme";
import "./styles.css";

initTheme();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <UiDensityProvider>
    <App />
  </UiDensityProvider>,
);
