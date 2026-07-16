import { Cable } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { FactList } from "@/components/shared";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type { LinkFactsReport } from "@/types";
import { formatTag } from "@/types";

interface LinkFactsPanelProps {
  loading?: boolean;
  report?: LinkFactsReport | null;
}

function issueVariant(severity: string) {
  if (severity === "critical") return "poor" as const;
  if (severity === "warning") return "caution" as const;
  return "info" as const;
}

function formatSpeed(report: LinkFactsReport) {
  const active = report.active;
  if (!active) return "Unknown";
  if (active.speed_mbps != null) return `${active.speed_mbps} Mbps`;
  if (active.raw_speed) return active.raw_speed;
  return "Unknown";
}

export function LinkFactsPanel({ loading, report }: LinkFactsPanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg" />;
  }

  if (!report) {
    return (
      <EmptyState
        icon={Cable}
        title="No link facts yet"
        description="Run a health check to read negotiated link speed, duplex, and Wi‑Fi vs Ethernet guidance."
      />
    );
  }

  const active = report.active;
  const label =
    active?.friendly_name ?? active?.name ?? report.adapters[0]?.name ?? "Unknown";

  return (
    <Card className="min-h-0 border-border/70 bg-card/80">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
            <Cable className="size-4" />
          </div>
          <CardTitle>Link</CardTitle>
        </div>
        <div className="flex flex-wrap justify-end gap-1">
          {active?.kind && <Badge variant="info">{formatTag(active.kind)}</Badge>}
          {active?.duplex && <Badge variant="info">{formatTag(active.duplex)}</Badge>}
        </div>
      </CardHeader>
      <CardContent className="min-h-0">
        <ScrollArea className="max-h-[28rem]">
          <div className="space-y-3 text-xs">
            <p className="text-sm font-medium">{report.summary}</p>
            <FactList
              rows={[
                { label: "Active adapter", value: label },
                { label: "Negotiated speed", value: formatSpeed(report) },
                {
                  label: "Media",
                  value: active?.media ?? "Not reported",
                },
                { label: "Source", value: report.source },
              ]}
            />
            {report.issues.length > 0 && (
              <div className="space-y-2">
                {report.issues.map((issue) => (
                  <div
                    key={`${issue.kind}-${issue.title}`}
                    className="rounded-lg border border-border/70 bg-muted/20 p-3"
                  >
                    <div className="mb-1 flex flex-wrap items-center gap-1">
                      <Badge variant={issueVariant(issue.severity)}>
                        {formatTag(issue.severity)}
                      </Badge>
                      <span className="text-sm font-medium">{issue.title}</span>
                    </div>
                    <p className="text-muted-foreground">{issue.message}</p>
                  </div>
                ))}
              </div>
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
