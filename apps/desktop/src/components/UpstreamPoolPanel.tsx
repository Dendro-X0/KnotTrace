import { ShieldAlert } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { UpstreamPoolClaim, UpstreamPoolProof } from "@/types";
import { formatTag } from "@/types";

interface UpstreamPoolPanelProps {
  loading?: boolean;
  proxyEnabled?: boolean;
  proof?: UpstreamPoolProof | null;
  compact?: boolean;
}

function claimVariant(claim: UpstreamPoolClaim) {
  if (claim === "upstream_pool_poor") return "poor" as const;
  if (claim === "active_path_impaired" || claim === "active_path_recurring") {
    return "caution" as const;
  }
  if (claim === "inconclusive") return "info" as const;
  return "active" as const;
}

function confidenceVariant(confidence: UpstreamPoolProof["confidence"]) {
  if (confidence === "high") return "active" as const;
  if (confidence === "medium") return "caution" as const;
  return "info" as const;
}

export function UpstreamPoolPanel({
  loading,
  proxyEnabled,
  proof,
  compact,
}: UpstreamPoolPanelProps) {
  if (loading) {
    return <Skeleton className={compact ? "h-36 rounded-lg" : "h-56 rounded-lg xl:col-span-2"} />;
  }

  if (!proxyEnabled) {
    return (
      <EmptyState
        icon={ShieldAlert}
        title="Upstream pool proof"
        description="Enable a system proxy to grade whether failures are the active path or a recurring upstream-pool problem."
      />
    );
  }

  if (!proof || proof.claim === "none") {
    return (
      <EmptyState
        icon={ShieldAlert}
        title="Upstream pool proof"
        description="Run a health check while a proxy is active. KnotTrace will not blame a whole pool from a single quiet snapshot."
      />
    );
  }

  return (
    <Card
      className={
        compact
          ? "min-h-0 border-border/70 bg-card/80"
          : "min-h-0 border-border/70 bg-card/80 xl:col-span-2"
      }
    >
      <CardHeader className={compact ? "pb-2" : undefined}>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <ShieldAlert className="size-4" />
            </div>
            <CardTitle>Upstream pool proof</CardTitle>
          </div>
          <div className="flex flex-wrap gap-1">
            <Badge variant={claimVariant(proof.claim)}>{formatTag(proof.claim)}</Badge>
            <Badge variant={confidenceVariant(proof.confidence)}>
              {proof.confidence} confidence
            </Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm font-medium leading-relaxed">{proof.title}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">{proof.summary}</p>
        <p className="text-xs leading-relaxed">{proof.action}</p>

        {!compact && (
          <>
            <div className="grid gap-2 text-xs sm:grid-cols-3">
              <div className="rounded-lg border border-border/70 bg-muted/20 p-2">
                Checks {proof.checks_considered}
              </div>
              <div className="rounded-lg border border-border/70 bg-muted/20 p-2">
                Impaired {proof.recurring_impaired_checks}
              </div>
              <div className="rounded-lg border border-border/70 bg-muted/20 p-2">
                Egress IPs {proof.distinct_egress_ips}
              </div>
            </div>

            {proof.proxy_only_failure_domains.length > 0 && (
              <p className="text-xs">
                Proxy-only failures:{" "}
                <span className="font-medium">{proof.proxy_only_failure_domains.join(", ")}</span>
              </p>
            )}
            {proof.intermittent_domains.length > 0 && (
              <p className="text-xs">
                Intermittent on proxy:{" "}
                <span className="font-medium">{proof.intermittent_domains.join(", ")}</span>
              </p>
            )}

            {proof.evidence.length > 0 && (
              <ul className="text-muted-foreground list-inside list-disc space-y-1 text-xs">
                {proof.evidence.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            )}
            {proof.not_proven.length > 0 && (
              <div>
                <p className="mb-1 text-xs font-medium">Not proven</p>
                <ul className="text-muted-foreground list-inside list-disc space-y-1 text-xs">
                  {proof.not_proven.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}
