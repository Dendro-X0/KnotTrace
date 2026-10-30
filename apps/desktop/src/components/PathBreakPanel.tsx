import { GitBranch } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { PathBreakReport } from "@/types";

interface PathBreakPanelProps {
  loading?: boolean;
  report?: PathBreakReport | null;
  compact?: boolean;
}

function confidenceVariant(confidence: PathBreakReport["confidence"]) {
  if (confidence === "high") return "active" as const;
  if (confidence === "medium") return "caution" as const;
  return "info" as const;
}

function hopVariant(hop: PathBreakReport["break_hop"], localOk: boolean) {
  if (hop === "none") return "active" as const;
  if (!localOk || hop === "local") return "caution" as const;
  return "poor" as const;
}

function label(value: string) {
  return value.replace(/_/g, " ");
}

export function PathBreakPanel({ loading, report, compact }: PathBreakPanelProps) {
  if (loading) {
    return <Skeleton className={compact ? "h-28 rounded-lg" : "h-40 rounded-lg"} />;
  }

  if (!report) {
    return (
      <EmptyState
        icon={GitBranch}
        title="Path break"
        description="Run a health check to classify Direct / proxy / VPN / Tor and where the path fails."
      />
    );
  }

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className={compact ? "pb-2" : undefined}>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <GitBranch className="size-4" />
            </div>
            <CardTitle className={compact ? "text-base" : undefined}>Path break</CardTitle>
          </div>
          <div className="flex flex-wrap items-center gap-1">
            <Badge variant="info">{label(report.active_path)}</Badge>
            <Badge variant={hopVariant(report.break_hop, report.local_ok)}>
              {label(report.break_hop)}
            </Badge>
            <Badge variant={confidenceVariant(report.confidence)}>
              {report.confidence} confidence
            </Badge>
            <Badge variant="info">{label(report.environment_class)}</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-sm font-medium leading-relaxed">{report.title}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">{report.summary}</p>
        {!compact && report.evidence.length > 0 && (
          <ul className="text-muted-foreground list-disc space-y-1 pl-4 text-xs">
            {report.evidence.slice(0, 4).map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
