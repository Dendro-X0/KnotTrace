import { GitCompareArrows } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type { ProxyPathReport, SiteReachErrorKind, SiteReachResult } from "@/types";
import { formatTag } from "@/types";

interface ProxyPathPanelProps {
  loading?: boolean;
  proxyEnabled?: boolean;
  report?: ProxyPathReport | null;
}

function confidenceVariant(confidence: ProxyPathReport["confidence"]) {
  if (confidence === "high") return "active" as const;
  if (confidence === "medium") return "caution" as const;
  return "info" as const;
}

function pathStatus(result: SiteReachResult) {
  if (result.success) {
    const latency =
      result.latency_ms != null ? ` · ${result.latency_ms.toFixed(0)} ms` : "";
    const status = result.status_code != null ? `HTTP ${result.status_code}` : "OK";
    return { label: `${status}${latency}`, variant: "active" as const };
  }

  const kind = result.error_kind ? formatTag(result.error_kind) : "failed";
  return { label: kind, variant: "poor" as const };
}

function errorKindLabel(kind: SiteReachErrorKind | null | undefined) {
  if (!kind) return null;
  return formatTag(kind);
}

export function ProxyPathPanel({ loading, proxyEnabled, report }: ProxyPathPanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg" />;
  }

  if (!proxyEnabled) {
    return (
      <EmptyState
        icon={GitCompareArrows}
        title="Proxy path comparison"
        description="Enable a system proxy to compare major-site reachability on proxy vs direct paths."
      />
    );
  }

  if (!report || report.comparisons.length === 0) {
    return (
      <EmptyState
        icon={GitCompareArrows}
        title="Proxy path report pending"
        description="Run a health check while a proxy is active to compare site access on proxy and direct paths."
      />
    );
  }

  return (
    <Card className="min-h-0 border-border/70 bg-card/80 xl:col-span-2">
      <CardHeader>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <GitCompareArrows className="size-4" />
            </div>
            <CardTitle>Proxy path report</CardTitle>
          </div>
          <div className="flex flex-wrap gap-1">
            <Badge variant={confidenceVariant(report.confidence)}>
              {report.confidence} confidence
            </Badge>
            {report.likely_provider_side && (
              <Badge variant="poor">Likely provider-side</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="min-h-0 space-y-3">
        <p className="text-muted-foreground text-sm leading-relaxed">{report.summary}</p>
        <p className="text-muted-foreground text-xs">
          {report.proxy_only_failure_count} proxy-only failure(s) · {report.proxy_failure_count}{" "}
          proxy failures · {report.direct_failure_count} direct failures across{" "}
          {report.checked_domains} sites
        </p>

        <ScrollArea className="min-h-0 max-h-[min(22rem,50vh)]">
          <div className="space-y-2">
            {report.comparisons.map((row) => {
              const proxyStatus = pathStatus(row.proxy);
              const directStatus = pathStatus(row.direct);
              return (
                <div
                  key={row.domain}
                  className="rounded-lg border border-border/70 bg-muted/20 p-3 text-xs"
                >
                  <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
                    <p className="font-medium">{row.domain}</p>
                    {row.proxy_only_failure && (
                      <Badge variant="caution">Proxy only</Badge>
                    )}
                  </div>
                  <div className="grid gap-2 sm:grid-cols-2">
                    <div className="rounded-md border border-border/60 bg-background/40 p-2">
                      <p className="text-muted-foreground mb-1 text-[0.68rem] uppercase tracking-wide">
                        Proxy path
                      </p>
                      <Badge variant={proxyStatus.variant}>{proxyStatus.label}</Badge>
                      {row.proxy.error && (
                        <p className="text-muted-foreground mt-1 line-clamp-2">{row.proxy.error}</p>
                      )}
                      {errorKindLabel(row.proxy.error_kind) && (
                        <p className="text-muted-foreground mt-1">
                          Class: {errorKindLabel(row.proxy.error_kind)}
                        </p>
                      )}
                    </div>
                    <div className="rounded-md border border-border/60 bg-background/40 p-2">
                      <p className="text-muted-foreground mb-1 text-[0.68rem] uppercase tracking-wide">
                        Direct path
                      </p>
                      <Badge variant={directStatus.variant}>{directStatus.label}</Badge>
                      {row.direct.error && (
                        <p className="text-muted-foreground mt-1 line-clamp-2">{row.direct.error}</p>
                      )}
                      {errorKindLabel(row.direct.error_kind) && (
                        <p className="text-muted-foreground mt-1">
                          Class: {errorKindLabel(row.direct.error_kind)}
                        </p>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
