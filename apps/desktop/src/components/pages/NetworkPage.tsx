import { Network, Radar } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { FactList } from "@/components/shared";
import { StabilityProbesPanel } from "@/components/StabilityProbesPanel";
import { ThroughputPanel } from "@/components/ThroughputPanel";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type { CompanionState } from "@/hooks/useCompanion";
import { formatTag } from "@/types";

interface NetworkPageProps {
  state: CompanionState;
}

export function NetworkPage({ state }: NetworkPageProps) {
  const env = state.report?.environment;
  const probe = state.report?.probe;
  const score = state.report?.score;
  const loading = state.bootstrapping && !state.report;

  return (
    <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 xl:grid-cols-2">
      <Card className="min-h-0 border-border/70 bg-card/80">
        <CardHeader>
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <Network className="size-4" />
            </div>
            <CardTitle>Environment</CardTitle>
          </div>
          <div className="flex flex-wrap justify-end gap-1">
            {(env?.tags ?? []).map((tag) => (
              <Badge key={tag} variant="info">
                {formatTag(tag)}
              </Badge>
            ))}
          </div>
        </CardHeader>
        <CardContent className="min-h-0">
          {loading ? (
            <div className="space-y-2">
              {Array.from({ length: 5 }).map((_, index) => (
                <Skeleton key={index} className="h-4 w-full" />
              ))}
            </div>
          ) : env ? (
            <ScrollArea className="max-h-[28rem]">
              <FactList
                className="pr-3"
                rows={[
                  { label: "Host", value: env.hostname },
                  { label: "Active interface", value: env.active_interface ?? "Unknown" },
                  { label: "Default gateway", value: env.default_gateway ?? "Not detected" },
                  {
                    label: "Proxy",
                    value: env.proxy.enabled
                      ? `${env.proxy.server ?? "enabled"} (${env.proxy.source})`
                      : "Off",
                  },
                  {
                    label: "DNS servers",
                    value:
                      env.dns_servers.map((server) => server.address).join(", ") ||
                      "System default",
                  },
                  {
                    label: "Tor",
                    value: env.tor?.detected
                      ? `${env.tor.socks_endpoint ?? "detected"} (${env.tor.source})${
                          env.tor.socks_reachable ? " · reachable" : " · unreachable"
                        }`
                      : "Not detected",
                  },
                ]}
              />
            </ScrollArea>
          ) : (
            <EmptyState
              icon={Network}
              title="No environment snapshot"
              description="Run a health check to capture interface, gateway, proxy, and DNS details."
            />
          )}
        </CardContent>
      </Card>

      <Card className="min-h-0 border-border/70 bg-card/80">
        <CardHeader>
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <Radar className="size-4" />
            </div>
            <CardTitle>Probe results</CardTitle>
          </div>
        </CardHeader>
        <CardContent className="min-h-0">
          {loading ? (
            <div className="space-y-2">
              {Array.from({ length: 4 }).map((_, index) => (
                <Skeleton key={index} className="h-4 w-full" />
              ))}
            </div>
          ) : probe ? (
            <ScrollArea className="max-h-[28rem]">
              <div className="pr-3">
                <FactList
                  rows={[
                    {
                      label: "Internet latency",
                      value: probe.internet
                        ? `${probe.internet.avg_ms.toFixed(0)} ms`
                        : "Unavailable",
                    },
                    {
                      label: "Gateway latency",
                      value: probe.gateway
                        ? `${probe.gateway.avg_ms.toFixed(0)} ms`
                        : "Not measured",
                    },
                    {
                      label: "DNS latency",
                      value:
                        probe.dns.find((sample) => sample.success)?.latency_ms.toFixed(0) ??
                        "Unavailable",
                    },
                    { label: "Probe duration", value: `${probe.duration_ms} ms` },
                  ]}
                />
                {score && score.reasons.length > 0 && (
                  <ul className="text-muted-foreground mt-3 list-disc space-y-1 pl-4 text-xs">
                    {score.reasons.map((reason) => (
                      <li key={reason}>{reason}</li>
                    ))}
                  </ul>
                )}
              </div>
            </ScrollArea>
          ) : (
            <EmptyState
              icon={Radar}
              title="No probe data yet"
              description="Latency and DNS measurements will appear here after your first health check."
            />
          )}
        </CardContent>
      </Card>

      <StabilityProbesPanel stability={state.report?.stability} />
      <ThroughputPanel state={state} />
    </div>
  );
}
