import { Globe2, ShieldAlert } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type { EgressReport, NetworkContextReport, NetworkRecommendations } from "@/types";
import { formatTag } from "@/types";

interface NetworkInsightsPanelProps {
  loading?: boolean;
  egress?: EgressReport | null;
  networkContext?: NetworkContextReport | null;
  recommendations?: NetworkRecommendations | null;
}

function riskBadgeVariant(risk: NetworkContextReport["risk_level"] | undefined) {
  if (risk === "high") return "poor" as const;
  if (risk === "moderate") return "caution" as const;
  return "active" as const;
}

export function NetworkInsightsPanel({
  loading,
  egress,
  networkContext,
  recommendations,
}: NetworkInsightsPanelProps) {
  if (loading) {
    return <Skeleton className="h-56 rounded-lg" />;
  }

  if (!egress && !networkContext && !recommendations) {
    return (
      <EmptyState
        icon={Globe2}
        title="No network insights yet"
        description="Run a health check to detect public IP, guest Wi-Fi risk, and guidance."
      />
    );
  }

  return (
    <Card className="min-h-0 border-border/70 bg-card/80">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
            <ShieldAlert className="size-4" />
          </div>
          <CardTitle>Network insights</CardTitle>
        </div>
        {networkContext && (
          <div className="flex flex-wrap justify-end gap-1">
            <Badge variant={riskBadgeVariant(networkContext.risk_level)}>
              {formatTag(networkContext.kind)}
            </Badge>
            <Badge variant="info">{formatTag(networkContext.risk_level)} risk</Badge>
          </div>
        )}
      </CardHeader>
      <CardContent className="min-h-0">
        <ScrollArea className="max-h-[28rem]">
          <div className="space-y-3 text-xs">
            {egress?.primary_ip && (
              <div className="rounded-lg border border-border/70 bg-muted/20 p-3">
                <p className="text-muted-foreground mb-1 text-[0.68rem] uppercase tracking-wide">
                  Public IP
                </p>
                <p className="font-mono text-sm">{egress.primary_ip}</p>
                <p className="text-muted-foreground mt-1">{egress.summary}</p>
              </div>
            )}

            {networkContext && (
              <div className="rounded-lg border border-border/70 bg-muted/20 p-3">
                <p className="mb-1 text-sm font-medium">{networkContext.summary}</p>
                {networkContext.signals.length > 0 && (
                  <ul className="text-muted-foreground list-disc space-y-1 pl-4">
                    {networkContext.signals.map((signal) => (
                      <li key={signal}>{signal}</li>
                    ))}
                  </ul>
                )}
                {networkContext.captive_portal.state !== "not_detected" && (
                  <p className="text-muted-foreground mt-2">
                    {networkContext.captive_portal.summary}
                  </p>
                )}
              </div>
            )}

            {(recommendations?.items ?? []).length > 0 ? (
              <div className="space-y-2">
                <p className="text-muted-foreground text-[0.68rem] uppercase tracking-wide">
                  Recommendations
                </p>
                {recommendations?.items.map((item) => (
                  <div
                    key={`${item.category}-${item.title}`}
                    className="rounded-lg border border-border/70 bg-muted/15 p-3"
                  >
                    <p className="text-sm font-medium">{item.title}</p>
                    <p className="text-muted-foreground mt-1">{item.message}</p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-muted-foreground">{recommendations?.summary}</p>
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
