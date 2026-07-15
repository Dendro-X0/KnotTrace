import { useCallback, useState } from "react";

import { UpdateBanner } from "@/components/UpdateBanner";
import { AppHeader } from "@/components/AppHeader";
import {
  GlobalSearch,
  useGlobalSearchShortcut,
} from "@/components/GlobalSearch";
import { ConnectPage } from "@/components/pages/ConnectPage";
import { DnsPage } from "@/components/pages/DnsPage";
import { NetworkPage } from "@/components/pages/NetworkPage";
import { OverviewPage } from "@/components/pages/OverviewPage";
import { ProtectPage } from "@/components/pages/ProtectPage";
import { Sidebar } from "@/components/Sidebar";
import { useCompanion } from "@/hooks/useCompanion";

export function App() {
  const state = useCompanion();
  const [searchOpen, setSearchOpen] = useState(false);
  const openSearch = useCallback(() => setSearchOpen(true), []);

  useGlobalSearchShortcut(openSearch);

  return (
    <div className="bg-background text-foreground grid min-h-dvh grid-cols-1 lg:h-dvh lg:grid-cols-[220px_minmax(0,1fr)] lg:overflow-hidden">
      <Sidebar
        page={state.page}
        onNavigate={state.navigate}
        onOpenSearch={openSearch}
        version={state.appVersion}
      />

      <div className="surface-panel grid min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-2 rounded-2xl border border-border/60 p-2 shadow-sm sm:gap-3 sm:p-3 lg:m-2 lg:mr-3 lg:mb-3 lg:overflow-hidden lg:p-4">
        <UpdateBanner
          update={state.updateCheck}
          checking={state.updateChecking}
          installing={state.updateInstalling}
          progress={state.updateProgress}
          onCheck={() => void state.checkForUpdates()}
          onInstall={() => void state.installUpdate()}
          onOpenRelease={() => void state.openUpdateRelease()}
        />
        <AppHeader state={state} />

        <main className="app-scroll min-h-0 overflow-x-hidden pb-2 lg:pb-0">
          {state.page === "overview" && <OverviewPage state={state} />}
          {state.page === "dns" && <DnsPage state={state} />}
          {state.page === "connect" && <ConnectPage state={state} />}
          {state.page === "protect" && <ProtectPage state={state} />}
          {state.page === "network" && <NetworkPage state={state} />}
        </main>
      </div>

      <GlobalSearch
        open={searchOpen}
        onOpenChange={setSearchOpen}
        onNavigate={state.navigate}
        onRunCheck={() => void state.runCheck()}
      />
    </div>
  );
}
