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
  /**
   * `hero` — tall Quick composition (legacy).
   * `focus` — denser Quick split (legacy).
   * `rail` — horizontal command-deck status strip.
   */
  emphasis?: "default" | "hero" | "focus" | "rail";
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
  const focus = emphasis === "focus";
  const rail = emphasis === "rail";
  const composed = hero || focus || rail;

  if (loading) {
    return (
      <Card className={cn("border-border/70 bg-card/60", rail && "border-0 bg-transparent shadow-none")}>
        <CardContent
          className={cn(
            "flex items-center gap-4",
            hero ? "py-6" : focus || rail ? "px-4 py-3.5" : "py-0",
          )}
        >
          <Skeleton className={cn("rounded-xl", hero ? "size-14" : "size-10")} />
          <div className="flex-1 space-y-2">
            <Skeleton className="h-5 w-36" />
            <Skeleton className="h-4 w-full max-w-md" />
          </div>
          {rail && <Skeleton className="hidden h-8 w-36 sm:block" />}
        </CardContent>
      </Card>
    );
  }

  const showDnsBadge =
    Boolean(dnsIntegrity) &&
    (composed ? dnsIntegrity?.state !== "ok" : Boolean(dnsIntegrity));
  const showProfileBadge = Boolean(checkProfile) && !composed;

  if (rail) {
    return (
      <div className={cn(statusCardClass(grade), "rounded-none border-0 bg-transparent shadow-none")}>
        <div className="flex flex-col gap-3 px-4 py-3.5 sm:px-5 lg:flex-row lg:items-center lg:gap-5">
          <div className="flex min-w-0 flex-1 items-start gap-3">
            <div className="bg-background/40 flex size-10 shrink-0 items-center justify-center rounded-xl border border-border/50 shadow-sm sm:size-11">
              <Icon className="size-5" />
            </div>
            <div className="min-w-0 flex-1 space-y-1">
              <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-1">
                <p className="text-lg font-semibold tracking-tight sm:text-xl">
                  {gradeHeadline(grade)}
                </p>
                {score != null && (
                  <Badge variant={grade ? gradeBadgeVariant(grade) : "muted"} className="normal-case">
                    {`${score}/100`}
                  </Badge>
                )}
                {showDnsBadge && dnsIntegrity && (
                  <Badge variant={integrityBadgeVariant(dnsIntegrity.state)} title={dnsIntegrity.summary}>
                    DNS {integrityLabel(dnsIntegrity.state)}
                  </Badge>
                )}
              </div>
              <p className="text-muted-foreground line-clamp-2 text-sm leading-snug">{summary}</p>
              {footer}
            </div>
          </div>

          {metrics && metrics.length > 0 && (
            <div className="grid grid-cols-3 gap-3 border-t border-border/40 pt-3 sm:max-w-xs sm:shrink-0 lg:border-t-0 lg:border-l lg:pt-0 lg:pl-5">
              {metrics.map((metric) => (
                <div key={metric.label} className="min-w-0">
                  <p className="text-muted-foreground text-[0.62rem] font-medium tracking-wide uppercase">
                    {metric.label}
                  </p>
                  <p className="text-sm font-semibold tabular-nums">{metric.value}</p>
                </div>
              ))}
            </div>
          )}

          {(primaryAction || secondaryAction) && (
            <div className="flex flex-wrap items-center gap-2 lg:shrink-0">
              {primaryAction && (
                <Button size="sm" className="hub-cta" onClick={primaryAction.onClick}>
                  {primaryAction.label}
                </Button>
              )}
              {secondaryAction && (
                <Button size="sm" variant="secondary" onClick={secondaryAction.onClick}>
                  {secondaryAction.label}
                </Button>
              )}
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <Card className={cn(statusCardClass(grade), composed && "h-full overflow-hidden")}>
      <CardContent
        className={cn(
          "flex flex-col",
          hero && "gap-4 px-5 py-5 sm:px-6 sm:py-6",
          focus && "gap-3 px-4 py-3.5 sm:px-5 sm:py-4",
          !composed && "flex-row items-start gap-3 py-0 sm:items-center sm:gap-4",
        )}
      >
        <div className={cn("flex min-w-0 items-start gap-3", !composed && "w-full sm:gap-4")}>
          <div
            className={cn(
              "bg-background/40 flex shrink-0 items-center justify-center border border-border/50 shadow-sm",
              hero && "size-14 rounded-2xl sm:size-16",
              focus && "size-11 rounded-xl sm:size-12",
              !composed && "size-10 rounded-xl sm:size-12",
            )}
          >
            <Icon
              className={cn(
                hero && "size-7 sm:size-8",
                focus && "size-5 sm:size-6",
                !composed && "size-5 sm:size-6",
              )}
            />
          </div>
          <div className={cn("min-w-0 flex-1", focus ? "space-y-1.5" : "space-y-2")}>
            <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-1">
              <p
                className={cn(
                  "font-semibold tracking-tight",
                  hero && "text-2xl sm:text-3xl",
                  focus && "text-xl sm:text-2xl",
                  !composed && "text-xl font-bold tracking-wide sm:text-2xl",
                )}
              >
                {composed ? gradeHeadline(grade) : grade ? grade.toUpperCase() : "—"}
              </p>
              {score != null && (
                <Badge
                  variant={grade ? gradeBadgeVariant(grade) : "muted"}
                  className={composed ? "normal-case" : undefined}
                >
                  {composed ? `${score}/100` : `Score ${score}/100`}
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
                hero && "max-w-2xl text-sm sm:text-base",
                focus && "line-clamp-2 max-w-xl text-sm",
                !composed && "text-sm text-foreground",
              )}
            >
              {summary}
            </p>
          </div>
        </div>

        {composed && metrics && metrics.length > 0 && (
          <div
            className={cn(
              "grid grid-cols-3 gap-2 border-t border-border/40",
              hero ? "max-w-md pt-4" : "max-w-sm pt-3",
            )}
          >
            {metrics.map((metric) => (
              <div key={metric.label} className="min-w-0">
                <p className="text-muted-foreground text-[0.62rem] font-medium tracking-wide uppercase">
                  {metric.label}
                </p>
                <p
                  className={cn(
                    "font-semibold tabular-nums",
                    hero ? "text-sm sm:text-base" : "text-sm",
                  )}
                >
                  {metric.value}
                </p>
              </div>
            ))}
          </div>
        )}

        {composed && (primaryAction || secondaryAction) && (
          <div className="flex flex-wrap items-center gap-2">
            {primaryAction && (
              <Button size={focus ? "sm" : "default"} className={focus ? "hub-cta" : undefined} onClick={primaryAction.onClick}>
                {primaryAction.label}
              </Button>
            )}
            {secondaryAction && (
              <Button
                size={focus ? "sm" : "default"}
                variant="secondary"
                onClick={secondaryAction.onClick}
              >
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
