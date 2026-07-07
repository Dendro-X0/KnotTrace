import { Loader2, RefreshCw } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import type { CompanionState } from "@/hooks/useCompanion";
import { formatReason, PAGE_TITLES } from "@/types";

interface AppHeaderProps {
  state: CompanionState;
}

export function AppHeader({ state }: AppHeaderProps) {
  const monitorActive = state.monitorStatus?.enabled ?? true;
  const lastReason = state.monitorStatus?.last_reason ?? "";
  const monitorText = state.monitorStatus
    ? `Checks every ${state.monitorStatus.poll_interval_secs}s · last trigger: ${formatReason(lastReason)}`
    : state.monitorError ?? "Starting background monitor...";

  return (
    <header className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
      <div className="min-w-0 space-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="text-lg font-semibold tracking-tight">{PAGE_TITLES[state.page]}</h2>
          <Badge variant={monitorActive ? "active" : "muted"}>
            Monitor {monitorActive ? "active" : "paused"}
          </Badge>
          {state.bootstrapping && (
            <Badge variant="info" className="gap-1">
              <Loader2 className="size-3 animate-spin" />
              Loading
            </Badge>
          )}
        </div>
        <p className="text-muted-foreground line-clamp-2 text-xs sm:line-clamp-none">{monitorText}</p>
        {state.report && (
          <p className="text-muted-foreground text-xs">
            Last checked {new Date(state.report.timestamp).toLocaleString()}
          </p>
        )}
        {state.checkError && <p className="text-destructive text-xs">{state.checkError}</p>}
      </div>

      <Separator className="lg:hidden" />

      <div className="flex shrink-0 flex-col items-stretch gap-2 min-[420px]:flex-row min-[420px]:items-center">
        <Button className="w-full min-[420px]:w-auto" onClick={() => void state.runCheck()} disabled={state.checking}>
          {state.checking ? (
            <>
              <Loader2 className="size-4 animate-spin" />
              <span className="sm:hidden">Checking</span>
              <span className="hidden sm:inline">Checking...</span>
            </>
          ) : (
            <>
              <RefreshCw className="size-4" />
              <span className="sm:hidden">Health check</span>
              <span className="hidden sm:inline">Run health check</span>
            </>
          )}
        </Button>
        <div className="flex items-center justify-between gap-2 rounded-xl border border-border/70 bg-card/60 px-3 py-2 shadow-sm min-[420px]:justify-start">
          <Switch
            id="monitor-toggle"
            checked={monitorActive}
            onCheckedChange={(checked) => {
              void state.setMonitorEnabled(checked).catch(() => undefined);
            }}
          />
          <Label htmlFor="monitor-toggle" className="text-muted-foreground text-xs">
            <span className="hidden min-[420px]:inline">Background monitor</span>
            <span className="min-[420px]:hidden">Monitor</span>
          </Label>
        </div>
      </div>
    </header>
  );
}
