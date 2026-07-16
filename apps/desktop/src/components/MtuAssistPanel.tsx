import { Cable } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { FactList } from "@/components/shared";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { MtuAssistReport, MtuAssistState } from "@/types";

interface MtuAssistPanelProps {
  loading?: boolean;
  report?: MtuAssistReport | null;
  state?: MtuAssistState | null;
  applying?: boolean;
  restoring?: boolean;
  error?: string | null;
  onApply?: () => void;
  onRestore?: () => void;
}

export function MtuAssistPanel({
  loading,
  report,
  state,
  applying,
  restoring,
  error,
  onApply,
  onRestore,
}: MtuAssistPanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg" />;
  }

  const available = state?.available ?? report?.available ?? false;
  if (!available) {
    return (
      <EmptyState
        icon={Cable}
        title="MTU assist"
        description={
          state?.platform_note ??
          report?.platform_note ??
          "Interface MTU clamp is available on Windows, macOS, and Linux."
        }
      />
    );
  }

  if (!report) {
    return (
      <EmptyState
        icon={Cable}
        title="No MTU snapshot"
        description="Run a health check. MTU assist only offers a clamp when fragmentation risk and a tunnel/proxy are both evidenced."
      />
    );
  }

  const repairActive = state?.repair_active ?? report.repair_active;
  const canRepair = (state?.can_repair ?? true) && report.can_repair && !repairActive;

  return (
    <Card className="min-h-0 border-border/70 bg-card/80">
      <CardHeader>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <Cable className="size-4" />
            </div>
            <CardTitle>MTU assist</CardTitle>
          </div>
          <div className="flex flex-wrap gap-1">
            {report.fragmentation_risk && <Badge variant="caution">Fragmentation risk</Badge>}
            {report.tunnel_evidenced && <Badge variant="info">Tunnel/proxy</Badge>}
            {repairActive && <Badge variant="active">Clamp active</Badge>}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm leading-relaxed">{report.summary}</p>
        <FactList
          rows={[
            { label: "Interface", value: report.interface_name ?? "—" },
            {
              label: "Current MTU",
              value: report.current_mtu != null ? String(report.current_mtu) : "—",
            },
            {
              label: "Recommended",
              value: report.recommended_mtu != null ? String(report.recommended_mtu) : "—",
            },
            {
              label: "Path estimate",
              value:
                report.estimated_path_mtu != null ? String(report.estimated_path_mtu) : "—",
            },
          ]}
        />
        <p className="text-muted-foreground text-xs leading-relaxed">{report.platform_note}</p>
        {error && <p className="text-destructive text-xs">{error}</p>}
        <div className="flex flex-wrap gap-2">
          <Button size="sm" disabled={!canRepair || applying} onClick={onApply}>
            {applying ? "Applying…" : "Apply clamp"}
          </Button>
          <Button
            size="sm"
            variant="secondary"
            disabled={!repairActive || restoring}
            onClick={onRestore}
          >
            {restoring ? "Restoring…" : "Restore"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
