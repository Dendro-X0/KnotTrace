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
import type { CompanionState } from "@/hooks/useCompanion";
import { cn } from "@/lib/utils";

interface ConnectPageProps {
  state: CompanionState;
}

function connectBadge(state: CompanionState) {
  const rec = state.connectRecommendation;
  if (!rec?.kernel) return { label: "Not found", variant: "muted" as const };
  if (rec.should_apply) return { label: "Recommended", variant: "recommended" as const };
  return { label: "Connected", variant: "active" as const };
}

export function ConnectPage({ state }: ConnectPageProps) {
  const badge = connectBadge(state);
  const rec = state.connectRecommendation;
  const switchTarget = rec?.recommended_switch;
  const loading = state.bootstrapping && !rec && !state.connectError;
  const canApply = !!rec?.kernel && !!rec.should_apply && !!switchTarget;
  const groups = rec?.groups ?? [];

  return (
    <FeaturePage
      title="Connect Assist"
      description={state.connectSummary}
      icon={Activity}
      badge={loading ? undefined : badge}
      error={state.connectError}
      loading={loading}
      footer={
        <Button
          onClick={() => void state.applyRecommendedConnect()}
          disabled={!canApply || state.connectApplying}
        >
          {state.connectApplying ? "Switching..." : "Switch to recommended node"}
        </Button>
      }
    >
      {loading ? (
        <Skeleton className="h-24 rounded-lg" />
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
      ) : groups.length === 0 ? (
        <EmptyState
          icon={PlugZap}
          title="No proxy controller found"
          description="Enable external-controller in Mihomo or sing-box, or save a custom API URL below."
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

      <Separator />

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
    </FeaturePage>
  );
}
