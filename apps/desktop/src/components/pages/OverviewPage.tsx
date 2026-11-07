import { Clock3, Globe, Router, Wifi } from "lucide-react";

import { BenchmarkPanel } from "@/components/BenchmarkPanel";
import { EmptyState } from "@/components/EmptyState";
import { MetricCard } from "@/components/MetricCard";
import { NetworkDiagnosisPanel } from "@/components/NetworkDiagnosisPanel";
import { PathBreakPanel } from "@/components/PathBreakPanel";
import { DiagnosticBundlePanel } from "@/components/DiagnosticBundlePanel";
import { IntegrityBoardPanel } from "@/components/IntegrityBoardPanel";
import { LocalExplainPanel } from "@/components/LocalExplainPanel";
import { CloudExplainPanel } from "@/components/CloudExplainPanel";
import { EgressTimelinePanel } from "@/components/EgressTimelinePanel";
import { NextStepsPanel } from "@/components/NextStepsPanel";
import { EnglishProbeNote } from "@/components/EnglishProbeNote";
import { PathExitJournal } from "@/components/PathExitJournal";
import { PathTruthTip } from "@/components/PathTruthTip";
import { QuickIntegrityTeaser } from "@/components/QuickIntegrityTeaser";
import { QuickPathTeaser } from "@/components/QuickPathTeaser";
import { OverviewScoreHero } from "@/components/OverviewScoreHero";
import { StatusHero } from "@/components/StatusHero";
import { TrendCharts } from "@/components/TrendCharts";
import { UpstreamPoolPanel } from "@/components/UpstreamPoolPanel";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { formatLatency, gradeBadgeVariant } from "@/components/shared";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { CompanionState } from "@/hooks/useCompanion";
import { useLocale } from "@/hooks/useLocale";
import {
  buildNextStepPlan,
  buildQuickHeroHeadline,
  buildQuickHeroSummary,
  integrityNeedsAttention,
  pathNeedsAttention,
  shouldSurfaceQuickNextCtas,
} from "@/lib/nextStepPlan";

interface OverviewPageProps {
  state: CompanionState;
}

export function OverviewPage({ state }: OverviewPageProps) {
  const { isQuick } = useUiDensity();
  const { t } = useLocale();
  const grade = state.report?.score.grade;
  const firstDnsSample = state.report?.probe.dns.find((sample) => sample.success);
  const loading = state.bootstrapping && !state.report;
  const lastReason = state.monitorStatus?.last_reason ?? "";
  const checkProfile = state.monitorStatus
    ? lastReason.startsWith("manual")
      ? "fast"
      : "full"
    : undefined;

  const quickStats = [
    {
      label: "Internet",
      value: formatLatency(state.report?.probe.internet?.avg_ms),
      icon: Wifi,
      tone: "sky" as const,
    },
    {
      label: "Gateway",
      value: formatLatency(state.report?.probe.gateway?.avg_ms, "Not measured"),
      icon: Router,
      tone: "amber" as const,
    },
    {
      label: "DNS",
      value: formatLatency(firstDnsSample?.latency_ms),
      icon: Globe,
      tone: "emerald" as const,
    },
  ];

  const nextPlan = isQuick ? buildNextStepPlan(state) : null;
  // Good-grade checks stay calm unless Path/Integrity/captive/bufferbloat need a look.
  const surfaceNextCtas = shouldSurfaceQuickNextCtas(state, nextPlan);
  const primaryNext = surfaceNextCtas ? nextPlan?.actions[0] : undefined;
  // One primary body CTA when attention surfaces — secondary stays in the evidence cards.
  const heroSummary = buildQuickHeroSummary(state);
  const heroSummaryText =
    heroSummary.kind === "key" ? t(heroSummary.key) : heroSummary.text;
  const heroShowsEnglishText = heroSummary.kind === "text";
  const heroHeadline = buildQuickHeroHeadline(state);
  const heroHeadlineText =
    heroHeadline.kind === "key" ? t(heroHeadline.key) : undefined;
  const showPathAttention = isQuick && pathNeedsAttention(state);
  const showIntegrityAttention = isQuick && integrityNeedsAttention(state);

  return (
    <div className={isQuick ? "flex w-full min-w-0 flex-col gap-2.5" : "flex flex-col gap-3"}>
      {isQuick ? (
        <div className="overview-hero-flow motion-stagger flex w-full min-w-0 flex-col gap-3">
          <PathTruthTip />
          <OverviewScoreHero
            loading={loading}
            grade={grade}
            score={state.report?.score.score}
            headline={heroHeadlineText}
            pathAttention={showPathAttention}
            integrityAttention={showIntegrityAttention}
            dnsIntegrity={state.report?.dns_integrity}
            summary={heroSummaryText}
            footer={heroShowsEnglishText ? <EnglishProbeNote variant="note" /> : undefined}
            metrics={
              loading
                ? undefined
                : quickStats.map((stat) => ({ label: stat.label, value: stat.value }))
            }
            primaryAction={
              primaryNext
                ? {
                    label: t(primaryNext.labelKey),
                    onClick: () => state.navigate(primaryNext.page),
                  }
                : undefined
            }
          />

          <section
            className="overview-category-grid grid items-stretch gap-2.5 md:grid-cols-3"
            aria-label="Path evidence"
          >
            <QuickPathTeaser
              loading={loading}
              report={state.report?.path_break}
              tunnelCompare={state.report?.tunnel_compare}
            />
            <QuickIntegrityTeaser
              loading={loading}
              report={state.report?.integrity}
              checking={state.checking}
              onRunCheck={() => void state.runCheck()}
              onOpenProtect={() => state.navigate("protect")}
            />
            <DiagnosticBundlePanel state={state} compact />
          </section>

          <section aria-label={t("pathJournal.title")}>
            <PathExitJournal loading={loading} points={state.trends} compact />
          </section>
        </div>
      ) : (
        <>
          <section className="grid gap-3 lg:grid-cols-[minmax(0,1.55fr)_minmax(11rem,0.85fr)]">
            <StatusHero
              loading={loading}
              grade={grade}
              score={state.report?.score.score}
              dnsIntegrity={state.report?.dns_integrity}
              checkProfile={checkProfile}
              summary={
                state.report?.diagnosis?.summary ??
                state.report?.score.summary ??
                (state.checkError
                  ? "Last check failed — use Retry in the banner above."
                  : "Click Run health check (or press Ctrl+K) to see connection health.")
              }
              footer={<EnglishProbeNote variant="note" />}
            />

            <div className="grid grid-cols-1 gap-3 min-[420px]:grid-cols-3 lg:grid-cols-1">
              {loading
                ? Array.from({ length: 3 }).map((_, index) => (
                    <Card key={index} className="py-3">
                      <CardContent className="py-0">
                        <Skeleton className="mb-2 h-3 w-16" />
                        <Skeleton className="h-5 w-20" />
                      </CardContent>
                    </Card>
                  ))
                : quickStats.map((stat) => (
                    <MetricCard
                      key={stat.label}
                      label={stat.label}
                      value={stat.value}
                      icon={stat.icon}
                      tone={stat.tone}
                    />
                  ))}
            </div>
          </section>

          <Card className="min-h-0">
            <CardHeader className="pb-2">
              <CardTitle>Diagnosis & benchmarks</CardTitle>
            </CardHeader>
            <CardContent className="grid min-h-0 items-start gap-3 md:grid-cols-2">
              {loading ? (
                <>
                  <Skeleton className="h-40 rounded-lg" />
                  <Skeleton className="h-40 rounded-lg" />
                </>
              ) : (
                <>
                  <div className="grid min-h-0 min-w-0 gap-3">
                    <NetworkDiagnosisPanel diagnosis={state.report?.diagnosis} />
                    <PathBreakPanel
                      loading={loading}
                      report={state.report?.path_break}
                      compact
                      checking={state.checking}
                      onRunCheck={() => void state.runCheck()}
                    />
                    <IntegrityBoardPanel
                      loading={loading}
                      report={state.report?.integrity}
                      compact
                      checking={state.checking}
                      onRunCheck={() => void state.runCheck()}
                    />
                    <EgressTimelinePanel
                      loading={loading}
                      report={state.report?.egress_timeline}
                      compact
                    />
                    <PathExitJournal loading={loading} points={state.trends} />
                    <LocalExplainPanel state={state} />
                    <CloudExplainPanel state={state} />
                    <DiagnosticBundlePanel state={state} />
                  </div>
                  <div className="grid min-h-0 gap-3">
                    <UpstreamPoolPanel
                      loading={loading}
                      proxyEnabled={state.report?.environment.proxy.enabled}
                      proof={state.report?.upstream_pool}
                      compact
                    />
                    <NextStepsPanel state={state} />
                    <BenchmarkPanel
                      snapshots={state.benchmarkSnapshots}
                      saving={state.benchmarkSaving}
                      error={state.benchmarkError}
                      onSave={state.saveBenchmark}
                      onDelete={state.deleteBenchmark}
                    />
                  </div>
                </>
              )}
            </CardContent>
          </Card>

          <Card className="min-h-0">
            <CardHeader className="pb-2">
              <CardTitle>Trends</CardTitle>
            </CardHeader>
            <CardContent>
              {loading ? (
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
                  {Array.from({ length: 4 }).map((_, index) => (
                    <Skeleton key={index} className="h-24 rounded-lg" />
                  ))}
                </div>
              ) : (
                <TrendCharts
                  points={state.trends}
                  error={state.trendsError}
                  checking={state.checking}
                  onRunCheck={() => void state.runCheck()}
                />
              )}
            </CardContent>
          </Card>

          <Card className="min-h-0">
            <CardHeader className="pb-2">
              <CardTitle>Recent checks</CardTitle>
            </CardHeader>
            <CardContent className="min-h-0">
              {loading ? (
                <div className="grid grid-cols-1 gap-2 min-[420px]:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-6">
                  {Array.from({ length: 6 }).map((_, index) => (
                    <Skeleton key={index} className="h-20 rounded-lg" />
                  ))}
                </div>
              ) : state.historyError ? (
                <EmptyState
                  icon={Clock3}
                  title="History unavailable"
                  description={state.historyError}
                />
              ) : state.history.length === 0 ? (
                <EmptyState
                  icon={Clock3}
                  title="No saved checks yet"
                  description="Run a health check to start building your connection history."
                  action={
                    <Button
                      size="sm"
                      onClick={() => void state.runCheck()}
                      disabled={state.checking}
                    >
                      Run health check
                    </Button>
                  }
                />
              ) : (
                <ScrollArea className="max-h-[24rem] lg:max-h-none">
                  <div className="grid grid-cols-1 gap-2 min-[420px]:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-6">
                    {state.history.map((item) => (
                      <div
                        key={item.timestamp}
                        className="rounded-lg border border-border/70 bg-muted/20 p-2.5"
                      >
                        <Badge variant={gradeBadgeVariant(item.score.grade)} className="mb-1">
                          {item.score.grade.toUpperCase()}
                        </Badge>
                        <p className="truncate text-xs">{item.score.summary}</p>
                        <p className="text-muted-foreground mt-1 text-[0.68rem]">
                          {new Date(item.timestamp).toLocaleString()}
                        </p>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              )}
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
