import { Route } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type { TunnelPathCompareReport, TunnelPathSample } from "@/types";
import { formatTag } from "@/types";

interface TunnelComparePanelProps {
  loading?: boolean;
  report?: TunnelPathCompareReport | null;
}

function pathBadge(path: TunnelPathSample) {
  if (!path.available) return { label: "Unavailable", variant: "poor" as const };
  if (path.failure_count > 0 && path.success_count === 0) {
    return { label: "Failing", variant: "poor" as const };
  }
  if (path.failure_count > 0) return { label: "Partial", variant: "caution" as const };
  return { label: "OK", variant: "active" as const };
}

export function TunnelComparePanel({ loading, report }: TunnelComparePanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg xl:col-span-2" />;
  }

  if (!report) {
    return (
      <EmptyState
        icon={Route}
        title="Tunnel compare"
        description="When Tor, VPN, or a system proxy is active, KnotTrace compares Direct vs encrypted paths and states honest expectations — it will not accelerate Tor."
      />
    );
  }

  return (
    <Card className="min-h-0 border-border/70 bg-card/80 xl:col-span-2">
      <CardHeader>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <Route className="size-4" />
            </div>
            <CardTitle>Tunnel compare</CardTitle>
          </div>
          <div className="flex flex-wrap gap-1">
            {report.tor_detected && (
              <Badge variant={report.tor_socks_reachable ? "info" : "poor"}>
                Tor {report.tor_socks_reachable ? "SOCKS up" : "SOCKS down"}
              </Badge>
            )}
            {report.vpn_detected && <Badge variant="caution">VPN</Badge>}
            {report.proxy_enabled && <Badge variant="info">Proxy</Badge>}
          </div>
        </div>
      </CardHeader>
      <CardContent className="min-h-0 space-y-3">
        <p className="text-sm font-medium leading-relaxed">{report.summary}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">{report.expectation}</p>

        {report.tor_only_failures.length > 0 && (
          <p className="text-xs">
            Tor-only failures:{" "}
            <span className="font-medium">{report.tor_only_failures.join(", ")}</span>
          </p>
        )}

        <ScrollArea className="max-h-[22rem]">
          <div className="grid gap-2 md:grid-cols-3">
            {report.paths.map((path) => {
              const badge = pathBadge(path);
              return (
                <div
                  key={path.kind}
                  className="rounded-lg border border-border/70 bg-muted/20 p-3 text-xs"
                >
                  <div className="mb-2 flex flex-wrap items-center justify-between gap-1">
                    <span className="text-sm font-medium">{path.label}</span>
                    <Badge variant={badge.variant}>{badge.label}</Badge>
                  </div>
                  <p className="text-muted-foreground mb-2">
                    {path.success_count}/{path.success_count + path.failure_count} sites ·{" "}
                    {path.median_latency_ms != null
                      ? `${path.median_latency_ms.toFixed(0)} ms median`
                      : "no latency"}
                  </p>
                  {path.egress_ip && (
                    <p className="font-mono text-[0.7rem]">egress {path.egress_ip}</p>
                  )}
                  {path.note && (
                    <p className="text-muted-foreground mt-2 leading-relaxed">{path.note}</p>
                  )}
                  {path.reachability.length > 0 && (
                    <ul className="mt-2 space-y-1">
                      {path.reachability.map((row) => (
                        <li key={row.domain} className="flex justify-between gap-2">
                          <span>{row.domain}</span>
                          <span className="text-muted-foreground">
                            {row.success
                              ? `${row.latency_ms?.toFixed(0) ?? "—"} ms`
                              : formatTag(row.error_kind ?? "failed")}
                          </span>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
