import { Clock3, Globe, Router, Wifi } from "lucide-react";

import { BenchmarkPanel } from "@/components/BenchmarkPanel";
import { EmptyState } from "@/components/EmptyState";
import { MetricCard } from "@/components/MetricCard";
import { NetworkDiagnosisPanel } from "@/components/NetworkDiagnosisPanel";
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
    <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 xl:grid-cols-2">
      <StatusHero
        loading={loading}
        grade={grade}
        score={state.report?.score.score}
        dnsIntegrity={state.report?.dns_integrity}
        summary={
          state.checkError ??
          state.report?.score.summary ??
          "Run a check to see connection health."
        }
      />

      <div className="grid grid-cols-3 gap-3">
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

      <Card className="min-h-0 xl:col-span-2">
        <CardHeader>
          <CardTitle>Diagnosis & benchmarks</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-3 lg:grid-cols-2">
          {loading ? (
            <>
              <Skeleton className="h-40 rounded-lg" />
              <Skeleton className="h-40 rounded-lg" />
            </>
          ) : (
            <>
              <NetworkDiagnosisPanel diagnosis={state.report?.diagnosis} />
              <BenchmarkPanel
                snapshots={state.benchmarkSnapshots}
                saving={state.benchmarkSaving}
                error={state.benchmarkError}
                onSave={state.saveBenchmark}
                onDelete={state.deleteBenchmark}
              />
            </>
          )}
        </CardContent>
      </Card>

      <Card className="min-h-0">
        <CardHeader>
          <CardTitle>Trends</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="grid grid-cols-1 gap-3 lg:grid-cols-3">
              {Array.from({ length: 3 }).map((_, index) => (
                <Skeleton key={index} className="h-24 rounded-lg" />
              ))}
            </div>
          ) : (
            <TrendCharts points={state.trends} error={state.trendsError} />
          )}
        </CardContent>
      </Card>

      <Card className="min-h-0 xl:col-span-2">
        <CardHeader>
          <CardTitle>Recent checks</CardTitle>
        </CardHeader>
        <CardContent className="min-h-0">
          {loading ? (
            <div className="grid grid-cols-2 gap-2 xl:grid-cols-6">
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
            <ScrollArea className="max-h-[24rem]">
              <div className="grid grid-cols-1 gap-2 pr-3 sm:grid-cols-2 xl:grid-cols-6">
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
