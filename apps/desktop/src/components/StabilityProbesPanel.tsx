import { Activity, Gauge } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { StabilityProbeResult } from "@/types";

interface StabilityProbesPanelProps {
  stability?: StabilityProbeResult | null;
}

function bufferbloatVariant(grade?: string) {
  if (grade === "severe" || grade === "moderate") return "caution" as const;
  if (grade === "mild") return "info" as const;
  return "active" as const;
}

export function StabilityProbesPanel({ stability }: StabilityProbesPanelProps) {
  if (!stability) {
    return (
      <Card className="border-border/70 bg-muted/10 xl:col-span-2">
        <CardHeader className="pb-2">
          <CardTitle className="text-base">Stability probes</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">
            Run a health check to measure bufferbloat and path MTU hints.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="border-border/70 bg-muted/10 xl:col-span-2">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-base">Stability probes</CardTitle>
          <Badge variant="muted">{stability.duration_ms} ms</Badge>
        </div>
      </CardHeader>
      <CardContent className="grid gap-3 md:grid-cols-2">
        <div className="rounded-lg border border-border/70 bg-background/40 p-3">
          <div className="mb-2 flex items-center gap-2">
            <Activity className="text-muted-foreground size-4" />
            <span className="text-sm font-medium">Bufferbloat</span>
            {stability.bufferbloat && (
              <Badge variant={bufferbloatVariant(stability.bufferbloat.grade)}>
                {stability.bufferbloat.grade}
              </Badge>
            )}
          </div>
          {stability.bufferbloat ? (
            <div className="text-muted-foreground space-y-1 text-xs">
              <p>{stability.bufferbloat.summary}</p>
              <p>
                Idle {stability.bufferbloat.idle_latency_ms.toFixed(0)} ms → loaded{" "}
                {stability.bufferbloat.loaded_latency_ms.toFixed(0)} ms (+
                {stability.bufferbloat.latency_delta_ms.toFixed(0)} ms)
              </p>
            </div>
          ) : (
            <p className="text-muted-foreground text-xs">Probe unavailable.</p>
          )}
        </div>

        <div className="rounded-lg border border-border/70 bg-background/40 p-3">
          <div className="mb-2 flex items-center gap-2">
            <Gauge className="text-muted-foreground size-4" />
            <span className="text-sm font-medium">Path MTU</span>
            {stability.mtu?.fragmentation_risk && (
              <Badge variant="caution">risk</Badge>
            )}
          </div>
          {stability.mtu ? (
            <div className="text-muted-foreground space-y-1 text-xs">
              <p>{stability.mtu.summary}</p>
              {stability.mtu.estimated_path_mtu != null && (
                <p>
                  Est. MTU {stability.mtu.estimated_path_mtu}
                  {stability.mtu.recommended_tcp_mss != null &&
                    ` · MSS ~${stability.mtu.recommended_tcp_mss}`}
                </p>
              )}
            </div>
          ) : (
            <p className="text-muted-foreground text-xs">Probe unavailable.</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
