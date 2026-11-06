import { Loader2, RefreshCw } from "lucide-react";

import { CheckFailureBanner } from "@/components/CheckFailureBanner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { useLocale } from "@/hooks/useLocale";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { CompanionState } from "@/hooks/useCompanion";
import { formatReason } from "@/types";

interface AppHeaderProps {
  state: CompanionState;
}

export function AppHeader({ state }: AppHeaderProps) {
  const { isQuick } = useUiDensity();
  const { t, locale } = useLocale();
  const monitorActive = state.monitorStatus?.enabled ?? true;
  const lastReason = state.monitorStatus?.last_reason ?? "";
  const monitorText = state.monitorStatus
    ? t("header.monitorDetail", {
        secs: state.monitorStatus.poll_interval_secs,
        reason: formatReason(lastReason),
      })
    : (state.monitorError ?? t("header.monitorStarting"));
  const pageTitle = t(`nav.${state.page}`);
  const checkedWhen = state.report
    ? new Date(state.report.timestamp).toLocaleString(locale === "zh-Hans" ? "zh-CN" : "en")
    : null;

  return (
    <div className="flex flex-col gap-2">
      <header className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="min-w-0 space-y-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-lg font-semibold tracking-tight">{pageTitle}</h2>
            {!isQuick && (
              <Badge variant={monitorActive ? "active" : "muted"}>
                {monitorActive ? t("header.monitorActive") : t("header.monitorPaused")}
              </Badge>
            )}
            {state.bootstrapping && (
              <Badge variant="info" className="gap-1">
                <Loader2 className="size-3 animate-spin" />
                {t("common.loading")}
              </Badge>
            )}
          </div>
          {isQuick ? (
            checkedWhen && (
              <p className="text-muted-foreground text-xs">
                {t("header.lastChecked", { when: checkedWhen })}
              </p>
            )
          ) : (
            <>
              <p className="text-muted-foreground line-clamp-2 text-xs sm:line-clamp-none">{monitorText}</p>
              {checkedWhen && (
                <p className="text-muted-foreground text-xs">
                  {t("header.lastChecked", { when: checkedWhen })}
                </p>
              )}
            </>
          )}
        </div>

        <Separator className="lg:hidden" />

        <div className="flex shrink-0 flex-col items-stretch gap-2 min-[420px]:flex-row min-[420px]:items-center">
          <Button className="w-full min-[420px]:w-auto" onClick={() => void state.runCheck()} disabled={state.checking}>
            {state.checking ? (
              <>
                <Loader2 className="size-4 animate-spin" />
                <span className="sm:hidden">{t("header.checkingShort")}</span>
                <span className="hidden sm:inline">{t("header.checking")}</span>
              </>
            ) : (
              <>
                <RefreshCw className="size-4" />
                <span className="sm:hidden">{t("header.runCheckShort")}</span>
                <span className="hidden sm:inline">{t("header.runCheck")}</span>
              </>
            )}
          </Button>
          {!isQuick && (
            <div className="flex items-center justify-between gap-2 rounded-xl border border-border/70 bg-card/60 px-3 py-2 shadow-sm min-[420px]:justify-start">
              <Switch
                id="monitor-toggle"
                checked={monitorActive}
                onCheckedChange={(checked) => {
                  void state.setMonitorEnabled(checked).catch(() => undefined);
                }}
              />
              <Label htmlFor="monitor-toggle" className="text-muted-foreground text-xs">
                {t("header.backgroundMonitor")}
              </Label>
            </div>
          )}
        </div>
      </header>
      {state.checkError && (
        <CheckFailureBanner
          message={state.checkError}
          onRetry={() => void state.runCheck()}
          checking={state.checking}
        />
      )}
    </div>
  );
}
