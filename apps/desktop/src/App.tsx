import { useCallback, useState } from "react";

import { UpdateBanner } from "@/components/UpdateBanner";
import { AppHeader } from "@/components/AppHeader";
import {
  GlobalSearch,
  useGlobalSearchShortcut,
} from "@/components/GlobalSearch";
import { ConnectPage } from "@/components/pages/ConnectPage";
import { CompanionHomePage } from "@/components/pages/CompanionHomePage";
import { DnsPage } from "@/components/pages/DnsPage";
import { NetworkPage } from "@/components/pages/NetworkPage";
import { OverviewPage } from "@/components/pages/OverviewPage";
import { ProtectPage } from "@/components/pages/ProtectPage";
import { SettingsPage } from "@/components/pages/SettingsPage";
import { Sidebar } from "@/components/Sidebar";
import { TitleBar } from "@/components/TitleBar";
import { useCompanion } from "@/hooks/useCompanion";

export function App() {
  const state = useCompanion();
  const [searchOpen, setSearchOpen] = useState(false);
  const openSearch = useCallback(() => setSearchOpen(true), []);

  useGlobalSearchShortcut(openSearch);

  if (state.capabilities?.companion_shell) {
    return (
      <div className="hub-shell text-foreground flex min-h-dvh flex-col">
        <TitleBar />
        <div className="hub-stage mx-auto flex min-h-0 w-full max-w-lg flex-1 flex-col gap-2 rounded-2xl border border-border/50 p-3 sm:m-2 sm:p-4">
          <UpdateBanner
            update={state.updateCheck}
            checking={state.updateChecking}
            installing={state.updateInstalling}
            progress={state.updateProgress}
            error={state.updateError}
            onCheck={() => void state.checkForUpdates()}
            onInstall={() => void state.installUpdate()}
            onOpenRelease={() => void state.openUpdateRelease()}
          />
          <main className="app-scroll min-h-0 flex-1 overflow-x-hidden">
            <CompanionHomePage state={state} />
          </main>
        </div>
      </div>
    );
  }

  return (
    <div className="hub-shell text-foreground flex min-h-dvh flex-col lg:h-dvh lg:overflow-hidden">
      <TitleBar />
      <div className="grid min-h-0 flex-1 grid-cols-1 lg:grid-cols-[220px_minmax(0,1fr)] lg:overflow-hidden">
        <Sidebar
          page={state.page}
          onNavigate={state.navigate}
          onOpenSearch={openSearch}
          version={state.appVersion}
        />

        <div className="hub-stage grid min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-2 rounded-2xl p-2 sm:gap-2.5 sm:p-3 lg:m-2 lg:mr-3 lg:mb-3 lg:overflow-hidden lg:p-3.5">
          <UpdateBanner
            update={state.updateCheck}
            checking={state.updateChecking}
            installing={state.updateInstalling}
            progress={state.updateProgress}
            error={state.updateError}
            onCheck={() => void state.checkForUpdates()}
            onInstall={() => void state.installUpdate()}
            onOpenRelease={() => void state.openUpdateRelease()}
          />
          <AppHeader state={state} />

          <main key={state.page} className="app-scroll motion-page-enter min-h-0 overflow-x-hidden pb-2 lg:pb-0">
            {state.page === "overview" && <OverviewPage state={state} />}
            {state.page === "dns" && <DnsPage state={state} />}
            {state.page === "connect" && <ConnectPage state={state} />}
            {state.page === "protect" && <ProtectPage state={state} />}
            {state.page === "network" && <NetworkPage state={state} />}
            {state.page === "settings" && <SettingsPage state={state} />}
          </main>
        </div>
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
