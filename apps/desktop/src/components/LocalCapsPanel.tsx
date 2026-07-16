import { Cpu } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { FactList } from "@/components/shared";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type { LocalCapsReport, LocalCapsState } from "@/types";
import { formatTag } from "@/types";

interface LocalCapsPanelProps {
  loading?: boolean;
  report?: LocalCapsReport | null;
  state?: LocalCapsState | null;
  applying?: boolean;
  restoring?: boolean;
  error?: string | null;
  onApply?: () => void;
  onRestore?: () => void;
}

function issueVariant(severity: string) {
  if (severity === "critical") return "poor" as const;
  if (severity === "warning") return "caution" as const;
  return "info" as const;
}

export function LocalCapsPanel({
  loading,
  report,
  state,
  applying,
  restoring,
  error,
  onApply,
  onRestore,
}: LocalCapsPanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg" />;
  }

  const available = state?.available ?? report?.available ?? false;
  if (!available) {
    return (
      <EmptyState
        icon={Cpu}
        title="Windows local caps"
        description={
          state?.platform_note ??
          report?.platform_note ??
          "TCP auto-tuning and NIC power repair are only available on Windows."
        }
      />
    );
  }

  if (!report) {
    return (
      <EmptyState
        icon={Cpu}
        title="No local caps snapshot"
        description="Run a health check to inspect Windows TCP auto-tuning and NIC power settings."
      />
    );
  }

  const repairActive = state?.repair_active ?? report.repair_active;
  const canRepair =
    (state?.can_repair ?? report.can_repair) &&
    !repairActive &&
    report.issues.length > 0;

  return (
    <Card className="min-h-0 border-border/70 bg-card/80">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
            <Cpu className="size-4" />
          </div>
          <CardTitle>Local caps</CardTitle>
        </div>
        <div className="flex flex-wrap justify-end gap-1">
          {repairActive ? (
            <Badge variant="active">Repair active</Badge>
          ) : report.issues.length > 0 ? (
            <Badge variant="caution">Attention</Badge>
          ) : (
            <Badge variant="info">OK</Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="min-h-0 space-y-3">
        <ScrollArea className="max-h-[22rem]">
          <div className="space-y-3 text-xs">
            <p className="text-sm font-medium">{report.summary}</p>
            <FactList
              rows={[
                {
                  label: "TCP auto-tuning",
                  value: report.tcp_autotuning_level ?? "Unknown",
                },
                {
                  label: "Adapter",
                  value: report.adapter_name ?? "Not detected",
                },
                {
                  label: "NIC power saving",
                  value:
                    report.adapter_power_saving == null
                      ? "Not reported"
                      : report.adapter_power_saving
                        ? "Allowed (may throttle)"
                        : "Disabled",
                },
              ]}
            />
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
        </ScrollArea>

        {error && <p className="text-destructive text-xs">{error}</p>}

        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Button size="sm" disabled={!canRepair || applying} onClick={onApply}>
            {applying ? "Applying..." : "Apply repair"}
          </Button>
          <Button
            size="sm"
            variant="secondary"
            disabled={!repairActive || restoring}
            onClick={onRestore}
          >
            {restoring ? "Restoring..." : "Restore"}
          </Button>
        </div>
        <p className="text-muted-foreground text-[0.68rem]">
          {report.platform_note} Opt-in only — never applied automatically.
        </p>
      </CardContent>
    </Card>
  );
}
