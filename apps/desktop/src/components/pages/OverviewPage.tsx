import { Clock3, Globe, Router, Wifi } from "lucide-react";

import { BenchmarkPanel } from "@/components/BenchmarkPanel";
import { EmptyState } from "@/components/EmptyState";
import { MetricCard } from "@/components/MetricCard";
import { NetworkDiagnosisPanel } from "@/components/NetworkDiagnosisPanel";
import { NextStepsPanel } from "@/components/NextStepsPanel";
import { StatusHero } from "@/components/StatusHero";
import { TrendCharts } from "@/components/TrendCharts";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { formatLatency, gradeBadgeVariant } from "@/components/shared";
import type { CompanionState } from "@/hooks/useCompanion";

interface OverviewPageProps {
  state: CompanionState;
}

export function OverviewPage({ state }: OverviewPageProps) {
  const grade = state.report?.score.grade;
  const firstDnsSample = state.report?.probe.dns.find((sample) => sample.success);
  const loading = state.bootstrapping && !state.report;
  const lastReason = state.monitorStatus?.last_reason ?? "";
  const checkProfile = state.monitorStatus
    ? (lastReason.startsWith("manual") ? "fast" : "full")
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

  return (
    <div className="flex flex-col gap-3">
      <section className="grid gap-3 lg:grid-cols-[minmax(0,1.55fr)_minmax(11rem,0.85fr)]">
        <StatusHero
          loading={loading}
          grade={grade}
          score={state.report?.score.score}
          dnsIntegrity={state.report?.dns_integrity}
          checkProfile={checkProfile}
          summary={
            state.checkError ??
            state.report?.diagnosis?.summary ??
            state.report?.score.summary ??
            "Run a check to see connection health."
          }
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
              <div className="min-h-0 min-w-0">
                <NetworkDiagnosisPanel diagnosis={state.report?.diagnosis} />
              </div>
              <div className="grid min-h-0 gap-3">
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
            <TrendCharts points={state.trends} error={state.trendsError} />
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
                <Button size="sm" onClick={() => void state.runCheck()} disabled={state.checking}>
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
    </div>
  );
}
