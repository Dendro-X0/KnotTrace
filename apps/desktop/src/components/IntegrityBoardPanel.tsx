import { ShieldAlert } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { IntegrityBoardReport } from "@/types";

interface IntegrityBoardPanelProps {
  loading?: boolean;
  report?: IntegrityBoardReport | null;
  compact?: boolean;
}

function levelVariant(level: IntegrityBoardReport["level"]) {
  if (level === "elevated") return "poor" as const;
  if (level === "caution") return "caution" as const;
  if (level === "info") return "info" as const;
  return "active" as const;
}

function severityVariant(severity: "info" | "warning" | "critical") {
  if (severity === "critical") return "untrusted" as const;
  if (severity === "warning") return "caution" as const;
  return "info" as const;
}

export function IntegrityBoardPanel({ loading, report, compact }: IntegrityBoardPanelProps) {
  if (loading) {
    return <Skeleton className={compact ? "h-28 rounded-lg" : "h-44 rounded-lg"} />;
  }

  if (!report) {
    return (
      <EmptyState
        icon={ShieldAlert}
        title="Integrity"
        description="Run a health check for exposure and integrity indicators (never a surveillance-safe badge)."
      />
    );
  }

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className={compact ? "pb-2" : undefined}>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <ShieldAlert className="size-4" />
            </div>
            <CardTitle className={compact ? "text-base" : undefined}>Integrity</CardTitle>
          </div>
          <div className="flex flex-wrap gap-1">
            <Badge variant={levelVariant(report.level)}>{report.level}</Badge>
            <Badge variant="info">{report.indicator_count} indicator(s)</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-sm font-medium leading-relaxed">{report.title}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">{report.summary}</p>
        <p className="text-xs leading-relaxed italic">{report.disclaimer}</p>
        {report.indicators.length > 0 && (
          <ul className="space-y-2">
            {report.indicators.slice(0, compact ? 3 : 6).map((item) => (
              <li
                key={`${item.kind}-${item.title}`}
                className="rounded-lg border border-border/70 bg-background/40 p-2.5 text-xs"
              >
                <div className="mb-1 flex items-center gap-2">
                  <Badge variant={severityVariant(item.severity)}>{item.severity}</Badge>
                  <span className="font-medium">{item.title}</span>
                </div>
                <p className="text-muted-foreground leading-relaxed">{item.message}</p>
              </li>
            ))}
          </ul>
        )}
        {compact && report.indicators.length > 3 && (
          <p className="text-muted-foreground text-xs">
            +{report.indicators.length - 3} more indicator(s) in the full report export.
          </p>
        )}
        {!compact && report.listeners?.scanned && (
          <p className="text-muted-foreground text-xs">{report.listeners.summary}</p>
        )}
        {!compact && report.gateway_arp?.observed && (
          <p className="text-muted-foreground text-xs">
            Gateway ARP {report.gateway_arp.gateway_ip}
            {report.gateway_arp.mac ? ` · ${report.gateway_arp.mac}` : ""} · {report.gateway_arp.source}
          </p>
        )}
        {!compact && report.cert_store?.observed && (
          <p className="text-muted-foreground text-xs">
            Trusted roots {report.cert_store.root_count} · fp {report.cert_store.fingerprint.slice(0, 16)}
            … · {report.cert_store.source}
          </p>
        )}
        {!compact && report.process_egress?.scanned && (
          <p className="text-muted-foreground text-xs">{report.process_egress.summary}</p>
        )}
        {!compact && report.autoruns?.scanned && (
          <p className="text-muted-foreground text-xs">{report.autoruns.summary}</p>
        )}
      </CardContent>
    </Card>
  );
}
