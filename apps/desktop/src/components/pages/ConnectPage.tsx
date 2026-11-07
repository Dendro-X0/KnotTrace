import { Activity, PlugZap } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { FeaturePage } from "@/components/FeaturePage";
import { FactList } from "@/components/shared";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { useLocale } from "@/hooks/useLocale";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { CompanionState } from "@/hooks/useCompanion";
import {
  isConnectUnsupported,
  shouldShowConnectSwitch,
} from "@/lib/connectUnsupported";
import { cn } from "@/lib/utils";

interface ConnectPageProps {
  state: CompanionState;
}

function connectBadge(
  state: CompanionState,
  unsupportedLabel: string,
): { label: string; variant: "muted" | "recommended" | "active" | "info" } {
  const rec = state.connectRecommendation;
  if (isConnectUnsupported(rec)) {
    return { label: unsupportedLabel, variant: "info" };
  }
  if (rec?.should_apply) return { label: "Recommended", variant: "recommended" };
  return { label: "Connected", variant: "active" };
}

export function ConnectPage({ state }: ConnectPageProps) {
  const { isQuick } = useUiDensity();
  const { t } = useLocale();
  const rec = state.connectRecommendation;
  const unsupported = isConnectUnsupported(rec);
  const showSwitch = shouldShowConnectSwitch(rec);
  const badge = connectBadge(state, t("connectPage.badgeUnsupported"));
  const switchTarget = rec?.recommended_switch;
  const loading = state.bootstrapping && !rec && !state.connectError;
  const groups = rec?.groups ?? [];
  const description = unsupported
    ? t("connectPage.unsupportedSummary")
    : state.connectSummary;

  return (
    <FeaturePage
      title="Connect"
      description={description}
      icon={Activity}
      badge={loading ? undefined : badge}
      error={state.connectError}
      loading={loading}
      footer={
        showSwitch ? (
          <Button
            onClick={() => void state.applyRecommendedConnect()}
            disabled={state.connectApplying}
          >
            {state.connectApplying ? "Switching..." : "Switch to recommended node"}
          </Button>
        ) : undefined
      }
    >
      {loading ? (
        <Skeleton className="h-24 rounded-lg" />
      ) : isQuick ? (
        rec?.kernel ? (
          <p className="text-muted-foreground text-xs">
            Controller detected:{" "}
            <span className="text-foreground font-medium">
              {rec.kernel.kind}
              {rec.kernel.version ? ` ${rec.kernel.version}` : ""}
            </span>
            {groups.length > 0 ? ` · ${groups.length} group(s)` : ""}
          </p>
        ) : null
      ) : (
        <FactList
          rows={[
            {
              label: "Kernel",
              value: rec?.kernel
                ? `${rec.kernel.kind}${rec.kernel.version ? ` (${rec.kernel.version})` : ""}`
                : "Not detected",
            },
            { label: "API", value: rec?.kernel?.api_base ?? "—" },
            { label: "Groups", value: String(groups.length) },
          ]}
        />
      )}

      {loading ? (
        <div className="grid gap-2">
          {Array.from({ length: 3 }).map((_, index) => (
            <Skeleton key={index} className="h-11 rounded-lg" />
          ))}
        </div>
      ) : unsupported ? (
        <div className="grid gap-3">
          <EmptyState
            icon={PlugZap}
            title={t("connectPage.unsupportedTitle")}
            description={t("connectPage.unsupportedSummary")}
          />
          <div className="rounded-lg border border-border/70 bg-muted/15 px-3 py-2.5">
            <p className="text-sm font-medium">{t("connectPage.measuredTitle")}</p>
            <ul className="text-muted-foreground mt-1.5 list-disc space-y-1 pl-4 text-xs leading-relaxed">
              <li>{t("connectPage.measuredPath")}</li>
              <li>{t("connectPage.measuredProxy")}</li>
              <li>{t("connectPage.measuredTunnel")}</li>
            </ul>
            <p className="text-muted-foreground mt-2 text-xs leading-relaxed">
              {t("connectPage.openNetwork")}
            </p>
          </div>
        </div>
      ) : groups.length === 0 ? (
        <EmptyState
          icon={PlugZap}
          title={t("connectPage.noGroupsTitle")}
          description={t("connectPage.noGroupsSummary")}
        />
      ) : (
        <ScrollArea className="min-h-0 flex-1">
          <ul className="grid gap-2">
            {groups.slice(0, 4).map((group) => {
              const best = group.members
                .filter((member) => member.alive)
                .sort((a, b) => (a.delay_ms ?? 9999) - (b.delay_ms ?? 9999))[0];
              const bestLabel = best
                ? `${best.name} (${best.delay_ms ?? "?"} ms)`
                : "no tested nodes";
              const highlight = switchTarget?.group_name === group.name;

              return (
                <li
                  key={group.name}
                  className={cn(
                    "flex items-center justify-between rounded-lg border px-3 py-2 text-sm",
                    highlight
                      ? "border-amber-500/30 bg-amber-500/10"
                      : "border-border/70 bg-muted/20",
                  )}
                >
                  <span>
                    {group.name} · {group.group_type}
                  </span>
                  <span className="text-muted-foreground text-right text-xs">
                    {group.current ?? "none"} → {bestLabel}
                  </span>
                </li>
              );
            })}
          </ul>
        </ScrollArea>
      )}

      {!isQuick && <Separator />}

      {!isQuick && (
        <details className="rounded-xl border border-border/70 bg-muted/15 p-3">
          <summary className="cursor-pointer text-sm font-medium">API settings</summary>
          <div className="mt-3 grid gap-3">
            <div className="grid gap-1.5">
              <Label htmlFor="connect-api-base">API URL</Label>
              <Input
                id="connect-api-base"
                value={state.connectApiBase}
                onChange={(event) => state.setConnectApiBase(event.target.value)}
                placeholder="http://127.0.0.1:9090"
              />
            </div>
            <div className="grid gap-1.5">
              <Label htmlFor="connect-secret">Secret (optional)</Label>
              <Input
                id="connect-secret"
                type="password"
                value={state.connectSecret}
                onChange={(event) => state.setConnectSecret(event.target.value)}
                placeholder="Bearer secret"
              />
            </div>
            <Button
              variant="secondary"
              onClick={() => void state.saveConnectConfig()}
              disabled={state.connectSaving}
            >
              {state.connectSaving ? "Saving..." : "Save API settings"}
            </Button>
          </div>
        </details>
      )}
    </FeaturePage>
  );
}
