import { Wrench } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { LocalRepairActionKind, LocalRepairReport } from "@/types";

interface LocalRepairPanelProps {
  loading?: boolean;
  report?: LocalRepairReport | null;
  runningAction?: LocalRepairActionKind | null;
  error?: string | null;
  onRun?: (action: LocalRepairActionKind) => void;
}

export function LocalRepairPanel({
  loading,
  report,
  runningAction,
  error,
  onRun,
}: LocalRepairPanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg" />;
  }

  if (!report) {
    return (
      <EmptyState
        icon={Wrench}
        title="Local repair"
        description="Run a health check. Flush DNS, renew DHCP, and restart adapter work even when the internet path is down."
      />
    );
  }

  if (!report.available) {
    return (
      <EmptyState
        icon={Wrench}
        title="Local repair unavailable"
        description={report.platform_note}
      />
    );
  }

  return (
    <Card className="min-h-0 h-full border-border/50">
      <CardHeader className="pb-1.5">
        <div className="flex flex-wrap items-center gap-2">
          <Wrench className="text-muted-foreground size-3.5" aria-hidden />
          <CardTitle className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Local repair
          </CardTitle>
          {report.local_problem_likely && (
            <Badge variant="caution" className="normal-case">
              Local issue likely
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm leading-relaxed">{report.summary}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">
          These fix local stack glitches — not your ISP or Steam CDN. Admin approval may be
          required.
        </p>
        {error && <p className="text-destructive text-xs">{error}</p>}
        <div className="space-y-2">
          {report.actions.map((action) => (
            <div
              key={action.kind}
              className="flex flex-col gap-2 rounded-lg border border-border/70 bg-muted/15 p-3 sm:flex-row sm:items-center sm:justify-between"
            >
              <div className="min-w-0 space-y-1">
                <div className="flex flex-wrap items-center gap-1">
                  <span className="text-sm font-medium">{action.title}</span>
                  {action.recommended && <Badge variant="caution">Recommended</Badge>}
                </div>
                <p className="text-muted-foreground text-xs leading-relaxed">{action.message}</p>
              </div>
              <Button
                size="sm"
                disabled={!action.available || runningAction != null}
                onClick={() => onRun?.(action.kind)}
              >
                {runningAction === action.kind ? "Running…" : "Run"}
              </Button>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
