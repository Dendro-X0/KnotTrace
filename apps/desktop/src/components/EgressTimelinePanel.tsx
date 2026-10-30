import { History } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { EgressTimelineReport } from "@/types";

interface EgressTimelinePanelProps {
  loading?: boolean;
  report?: EgressTimelineReport | null;
  compact?: boolean;
}

function claimVariant(claim: EgressTimelineReport["claim"]) {
  if (claim === "churning" || claim === "unstable") return "poor" as const;
  if (claim === "ip_changed") return "caution" as const;
  if (claim === "stable") return "active" as const;
  return "info" as const;
}

function label(value: string) {
  return value.replace(/_/g, " ");
}

export function EgressTimelinePanel({ loading, report, compact }: EgressTimelinePanelProps) {
  if (loading) {
    return <Skeleton className={compact ? "h-28 rounded-lg" : "h-44 rounded-lg"} />;
  }

  if (!report) {
    return (
      <EmptyState
        icon={History}
        title="Egress timeline"
        description="Run checks over time to see public IP stability and churn from local history."
      />
    );
  }

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className={compact ? "pb-2" : undefined}>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <History className="size-4" />
            </div>
            <CardTitle className={compact ? "text-base" : undefined}>Egress timeline</CardTitle>
          </div>
          <div className="flex flex-wrap gap-1">
            <Badge variant={claimVariant(report.claim)}>{label(report.claim)}</Badge>
            <Badge variant="info">{report.confidence} confidence</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-sm font-medium leading-relaxed">{report.title}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">{report.summary}</p>
        {(report.current_ip || report.previous_ip) && (
          <p className="text-xs">
            {report.current_ip && (
              <>
                Current: <span className="font-medium">{report.current_ip}</span>
              </>
            )}
            {report.previous_ip && (
              <>
                {report.current_ip ? " · " : null}
                Previous: <span className="font-medium">{report.previous_ip}</span>
              </>
            )}
          </p>
        )}
        {!compact && report.events.length > 0 && (
          <ul className="text-muted-foreground list-disc space-y-1 pl-4 text-xs">
            {report.events.slice(0, 5).map((event) => (
              <li key={`${event.at}-${event.kind}-${event.detail}`}>
                {label(event.kind)}: {event.detail}
              </li>
            ))}
          </ul>
        )}
        {!compact && (
          <p className="text-xs leading-relaxed">{report.action}</p>
        )}
      </CardContent>
    </Card>
  );
}
