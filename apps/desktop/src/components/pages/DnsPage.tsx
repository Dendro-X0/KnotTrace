import { Globe, Server } from "lucide-react";

import { DnsIntegrityPanel } from "@/components/DnsIntegrityPanel";
import { DnsIntegritySettingsEditor } from "@/components/DnsIntegritySettingsEditor";
import { EmptyState } from "@/components/EmptyState";
import { FeaturePage } from "@/components/FeaturePage";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { CompanionState } from "@/hooks/useCompanion";
import { cn } from "@/lib/utils";

interface DnsPageProps {
  state: CompanionState;
}

function dnsBadge(state: CompanionState) {
  if (state.dnsState?.active) return { label: "Active", variant: "active" as const };
  if (state.dnsRecommendation?.should_apply) {
    return { label: "Recommended", variant: "recommended" as const };
  }
  return { label: "No change", variant: "muted" as const };
}

export function DnsPage({ state }: DnsPageProps) {
  const { isQuick } = useUiDensity();
  const badge = dnsBadge(state);
  const loading = state.bootstrapping && !state.dnsRecommendation && !state.dnsError;
  const canApply =
    state.dnsState?.can_apply &&
    !state.dnsState.active &&
    !!state.dnsRecommendation?.recommended &&
    state.dnsRecommendation.should_apply;
  const candidates = state.dnsRecommendation?.candidates ?? [];

  return (
    <FeaturePage
      title="DNS Assist"
      description={
        state.dnsSummary ||
        "Compare public resolvers and apply a recommended DNS when it helps."
      }
      icon={Globe}
      badge={loading ? undefined : badge}
      error={state.dnsError}
      loading={loading}
      footer={
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Button
            className={cn(canApply && "hub-cta")}
            onClick={() => void state.applyRecommendedDns()}
            disabled={!canApply || state.dnsApplying}
          >
            {state.dnsApplying ? "Applying…" : "Apply recommended DNS"}
          </Button>
          <Button
            variant="secondary"
            onClick={() => void state.restoreDns()}
            disabled={!state.dnsState?.can_apply || !state.dnsState.active || state.dnsRestoring}
          >
            {state.dnsRestoring ? "Restoring…" : "Restore original DNS"}
          </Button>
        </div>
      }
    >
      {loading ? (
        <div className="grid gap-2">
          {Array.from({ length: 4 }).map((_, index) => (
            <Skeleton key={index} className="h-11 rounded-xl" />
          ))}
        </div>
      ) : candidates.length === 0 ? (
        <EmptyState
          icon={Server}
          title="No resolver benchmarks yet"
          description="Run a health check to benchmark public DNS resolvers against your current setup."
          action={
            <Button
              size="sm"
              className="hub-cta"
              onClick={() => void state.runCheck()}
              disabled={state.checking}
            >
              {state.checking ? "Checking…" : "Run health check"}
            </Button>
          }
        />
      ) : (
        <ScrollArea className="min-h-0 flex-1">
          <ul className="grid gap-2">
            {candidates.map((candidate) => {
              const recommended =
                state.dnsRecommendation?.recommended?.resolver === candidate.resolver;
              return (
                <li
                  key={candidate.resolver}
                  className={cn(
                    "flex items-center justify-between rounded-xl border px-3 py-2.5 text-sm",
                    recommended
                      ? "border-emerald-500/30 bg-emerald-500/10"
                      : "border-border/50 bg-muted/15",
                    !candidate.success && "opacity-60",
                  )}
                >
                  <span className="min-w-0 truncate">
                    {candidate.label}{" "}
                    <span className="text-muted-foreground">({candidate.resolver})</span>
                  </span>
                  <span className="text-muted-foreground shrink-0 tabular-nums">
                    {candidate.success ? `${candidate.latency_ms.toFixed(0)} ms` : "unavailable"}
                  </span>
                </li>
              );
            })}
          </ul>
        </ScrollArea>
      )}

      {state.dnsState?.platform_note && (
        <p className="text-muted-foreground text-xs leading-relaxed">
          {state.dnsState.platform_note}
        </p>
      )}

      <DnsIntegrityPanel integrity={state.report?.dns_integrity} />

      {!isQuick && (
        <DnsIntegritySettingsEditor
          domains={state.integritySettings?.verification_domains ?? []}
          saving={state.integritySettingsSaving}
          error={state.integritySettingsError}
          onSave={state.saveIntegritySettings}
        />
      )}
    </FeaturePage>
  );
}
