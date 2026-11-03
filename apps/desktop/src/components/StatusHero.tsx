import type { ReactNode } from "react";
import { AlertTriangle, CheckCircle2, HelpCircle, XCircle } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
  gradeBadgeVariant,
  integrityBadgeVariant,
  integrityLabel,
  statusCardClass,
} from "@/components/shared";
import { cn } from "@/lib/utils";
import type { DnsIntegrityStatus, HealthGrade } from "@/types";

interface StatusHeroProps {
  grade?: HealthGrade;
  summary: string;
  score?: number;
  loading?: boolean;
  dnsIntegrity?: DnsIntegrityStatus | null;
  checkProfile?: "fast" | "full";
  /** Full-bleed Quick composition: larger type, fewer badges, room for metrics + CTA. */
  emphasis?: "default" | "hero";
  metrics?: Array<{ label: string; value: string }>;
  primaryAction?: { label: string; onClick: () => void };
  secondaryAction?: { label: string; onClick: () => void };
  footer?: ReactNode;
}

function gradeIcon(grade?: HealthGrade) {
  if (grade === "good") return CheckCircle2;
  if (grade === "fair") return AlertTriangle;
  if (grade === "poor") return XCircle;
  return HelpCircle;
}

function gradeHeadline(grade?: HealthGrade) {
  if (grade === "good") return "Looking good";
  if (grade === "fair") return "Needs attention";
  if (grade === "poor") return "Connection issues";
  return "Not checked yet";
}

export function StatusHero({
  grade,
  summary,
  score,
  loading,
  dnsIntegrity,
  checkProfile,
  emphasis = "default",
  metrics,
  primaryAction,
  secondaryAction,
  footer,
}: StatusHeroProps) {
  const Icon = gradeIcon(grade);
  const hero = emphasis === "hero";

  if (loading) {
    return (
      <Card className="border-border/70 bg-card/60">
        <CardContent className={cn("flex items-center gap-4", hero ? "py-6" : "py-0")}>
          <Skeleton className={cn("rounded-xl", hero ? "size-14" : "size-12")} />
          <div className="flex-1 space-y-2">
            <Skeleton className="h-7 w-40" />
            <Skeleton className="h-4 w-full max-w-lg" />
            {hero && <Skeleton className="h-9 w-48" />}
          </div>
        </CardContent>
      </Card>
    );
  }

  const showDnsBadge =
    Boolean(dnsIntegrity) && (hero ? dnsIntegrity?.state !== "ok" : Boolean(dnsIntegrity));
  const showProfileBadge = Boolean(checkProfile) && !hero;

  return (
    <Card className={cn(statusCardClass(grade), hero && "overflow-hidden")}>
      <CardContent
        className={cn(
          "flex flex-col gap-4",
          hero ? "px-5 py-5 sm:px-6 sm:py-6" : "flex-row items-start gap-3 py-0 sm:items-center sm:gap-4",
        )}
      >
        <div className={cn("flex min-w-0 items-start gap-3 sm:gap-4", !hero && "w-full")}>
          <div
            className={cn(
              "bg-background/40 flex shrink-0 items-center justify-center rounded-2xl border border-border/50 shadow-sm",
              hero ? "size-14 sm:size-16" : "size-10 sm:size-12 rounded-xl",
            )}
          >
            <Icon className={cn(hero ? "size-7 sm:size-8" : "size-5 sm:size-6")} />
          </div>
          <div className="min-w-0 flex-1 space-y-2">
            <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
              <p
                className={cn(
                  "font-semibold tracking-tight",
                  hero ? "text-2xl sm:text-3xl" : "text-xl font-bold tracking-wide sm:text-2xl",
                )}
              >
                {hero ? gradeHeadline(grade) : grade ? grade.toUpperCase() : "—"}
              </p>
              {score != null && (
                <Badge variant={grade ? gradeBadgeVariant(grade) : "muted"} className={hero ? "normal-case" : undefined}>
                  {hero ? `${score}/100` : `Score ${score}/100`}
                </Badge>
              )}
              {showDnsBadge && dnsIntegrity && (
                <Badge variant={integrityBadgeVariant(dnsIntegrity.state)} title={dnsIntegrity.summary}>
                  DNS {integrityLabel(dnsIntegrity.state)}
                </Badge>
              )}
              {showProfileBadge && checkProfile && (
                <Badge variant="info">{checkProfile.toUpperCase()} check</Badge>
              )}
            </div>
            <p
              className={cn(
                "text-muted-foreground leading-relaxed",
                hero ? "max-w-2xl text-sm sm:text-base" : "text-sm text-foreground",
              )}
            >
              {summary}
            </p>
          </div>
        </div>

        {hero && metrics && metrics.length > 0 && (
          <div className="grid grid-cols-3 gap-2 border-t border-border/40 pt-4 sm:max-w-md">
            {metrics.map((metric) => (
              <div key={metric.label} className="min-w-0">
                <p className="text-muted-foreground text-[0.65rem] font-medium tracking-wide uppercase">
                  {metric.label}
                </p>
                <p className="text-sm font-semibold tabular-nums sm:text-base">{metric.value}</p>
              </div>
            ))}
          </div>
        )}

        {hero && (primaryAction || secondaryAction) && (
          <div className="flex flex-wrap items-center gap-2">
            {primaryAction && (
              <Button size="default" onClick={primaryAction.onClick}>
                {primaryAction.label}
              </Button>
            )}
            {secondaryAction && (
              <Button size="default" variant="secondary" onClick={secondaryAction.onClick}>
                {secondaryAction.label}
              </Button>
            )}
          </div>
        )}

        {footer}
      </CardContent>
    </Card>
  );
}
