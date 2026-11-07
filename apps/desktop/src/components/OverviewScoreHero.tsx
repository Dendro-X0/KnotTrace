import type { ReactNode } from "react";
import { AlertTriangle, CheckCircle2, HelpCircle, XCircle } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { MetricStrip } from "@/components/MetricStrip";
import {
  gradeBadgeVariant,
  integrityBadgeVariant,
  integrityLabel,
  statusCardClass,
} from "@/components/shared";
import { cn } from "@/lib/utils";
import type { DnsIntegrityStatus, HealthGrade } from "@/types";

interface OverviewScoreHeroProps {
  grade?: HealthGrade;
  score?: number;
  /** Overrides grade-derived headline when Path/Integrity need attention. */
  headline?: string;
  summary: string;
  loading?: boolean;
  dnsIntegrity?: DnsIntegrityStatus | null;
  pathAttention?: boolean;
  integrityAttention?: boolean;
  metrics?: Array<{ label: string; value: string; icon?: ReactNode }>;
  primaryAction?: { label: string; onClick: () => void };
  secondaryAction?: { label: string; onClick: () => void };
  footer?: ReactNode;
}

function gradeIcon(grade?: HealthGrade, attention?: boolean) {
  if (attention) return AlertTriangle;
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

function ringColor(grade?: HealthGrade) {
  if (grade === "good") return "stroke-emerald-400";
  if (grade === "fair") return "stroke-amber-400";
  if (grade === "poor") return "stroke-rose-400";
  return "stroke-primary";
}

function ScoreRing({
  score,
  grade,
}: {
  score: number | undefined;
  grade?: HealthGrade;
}) {
  const value = score == null ? 0 : Math.max(0, Math.min(100, score));
  const radius = 54;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (value / 100) * circumference;

  const glow =
    grade === "good"
      ? "bg-emerald-400/35"
      : grade === "fair"
        ? "bg-amber-400/30"
        : grade === "poor"
          ? "bg-rose-400/30"
          : "bg-primary/30";

  return (
    // Extra padding so soft glow is not clipped by the hero card edges.
    <div className="relative mx-auto flex size-[11.5rem] items-center justify-center p-4 sm:size-[12.5rem]">
      <div
        aria-hidden
        className={cn(
          "pointer-events-none absolute inset-[12%] rounded-full blur-2xl",
          glow,
        )}
      />
      <div className="relative flex size-[9.5rem] items-center justify-center sm:size-[10.5rem]">
        <svg viewBox="0 0 128 128" className="size-full -rotate-90" aria-hidden>
          <circle
            cx="64"
            cy="64"
            r={radius}
            fill="none"
            className="stroke-border/50"
            strokeWidth="8"
          />
          <circle
            cx="64"
            cy="64"
            r={radius}
            fill="none"
            className={cn(
              ringColor(grade),
              "transition-[stroke-dashoffset] duration-700 ease-out",
            )}
            strokeWidth="8"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={offset}
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <p className="text-3xl font-semibold tracking-tight tabular-nums sm:text-4xl">
            {score == null ? "—" : value}
          </p>
          <p className="text-muted-foreground text-[0.65rem] font-medium tracking-[0.14em] uppercase">
            / 100
          </p>
        </div>
      </div>
    </div>
  );
}

export function OverviewScoreHero({
  grade,
  score,
  headline,
  summary,
  loading,
  dnsIntegrity,
  pathAttention = false,
  integrityAttention = false,
  metrics = [],
  primaryAction,
  secondaryAction,
  footer,
}: OverviewScoreHeroProps) {
  const attention = pathAttention || integrityAttention;
  const Icon = gradeIcon(grade, attention);
  const title = headline?.trim() || gradeHeadline(grade);

  if (loading) {
    return (
      <Card className="border-border/60">
        <CardContent className="space-y-4 py-6">
          <div className="flex flex-col items-center gap-2">
            <Skeleton className="h-7 w-48 rounded-lg" />
            <Skeleton className="h-4 w-72 max-w-full rounded-lg" />
          </div>
          <Skeleton className="mx-auto size-36 rounded-full" />
          <div className="mx-auto grid w-full max-w-lg grid-cols-3 gap-2">
            <Skeleton className="h-14 rounded-xl" />
            <Skeleton className="h-14 rounded-xl" />
            <Skeleton className="h-14 rounded-xl" />
          </div>
        </CardContent>
      </Card>
    );
  }

  const showDnsBadge = Boolean(dnsIntegrity) && dnsIntegrity?.state !== "ok";

  return (
    <div className="flex flex-col gap-2.5">
      <Card className={cn(statusCardClass(grade), "overflow-visible")}>
        <CardContent className="space-y-3 overflow-visible px-4 py-5 sm:px-6 sm:py-6">
          <div className="flex flex-col items-center gap-1 text-center">
            <div className="flex flex-wrap items-center justify-center gap-2">
              <Icon
                className={cn(
                  "size-5",
                  attention ? "text-amber-500 dark:text-amber-300" : "text-foreground/80",
                )}
              />
              <p className="text-balance text-xl font-semibold tracking-tight sm:text-2xl">
                {title}
              </p>
              {score != null && (
                <Badge
                  variant={grade ? gradeBadgeVariant(grade) : "muted"}
                  className="normal-case"
                  title="Measured check score (latency / reachability) — not a path or Protect verdict"
                >
                  {score}/100
                </Badge>
              )}
              {pathAttention && (
                <Badge variant="caution" className="normal-case">
                  Path
                </Badge>
              )}
              {integrityAttention && (
                <Badge variant="caution" className="normal-case">
                  Protect
                </Badge>
              )}
              {showDnsBadge && dnsIntegrity && (
                <Badge
                  variant={integrityBadgeVariant(dnsIntegrity.state)}
                  title={dnsIntegrity.summary}
                  className="normal-case"
                >
                  DNS {integrityLabel(dnsIntegrity.state)}
                </Badge>
              )}
            </div>
            <p className="text-muted-foreground max-w-xl text-sm leading-snug text-pretty">
              {summary}
            </p>
            {footer}
          </div>

          <ScoreRing score={score} grade={grade} />

          {metrics.length > 0 && <MetricStrip metrics={metrics} />}
        </CardContent>
      </Card>

      {(primaryAction || secondaryAction) && (
        <div className="flex flex-wrap items-center justify-center gap-2 pt-0.5">
          {primaryAction && (
            <Button className="hub-cta min-w-[11rem]" onClick={primaryAction.onClick}>
              {primaryAction.label}
            </Button>
          )}
          {secondaryAction && (
            <Button variant="secondary" className="min-w-[9rem]" onClick={secondaryAction.onClick}>
              {secondaryAction.label}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
