import { AlertTriangle, CheckCircle2, HelpCircle, XCircle } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { gradeBadgeVariant, integrityBadgeVariant, integrityLabel, statusCardClass } from "@/components/shared";
import type { DnsIntegrityStatus, HealthGrade } from "@/types";

interface StatusHeroProps {
  grade?: HealthGrade;
  summary: string;
  score?: number;
  loading?: boolean;
  dnsIntegrity?: DnsIntegrityStatus | null;
}

function gradeIcon(grade?: HealthGrade) {
  if (grade === "good") return CheckCircle2;
  if (grade === "fair") return AlertTriangle;
  if (grade === "poor") return XCircle;
  return HelpCircle;
}

export function StatusHero({ grade, summary, score, loading, dnsIntegrity }: StatusHeroProps) {
  const Icon = gradeIcon(grade);

  if (loading) {
    return (
      <Card className="border-border/70 bg-card/60">
        <CardContent className="flex items-center gap-4 py-0">
          <Skeleton className="size-12 rounded-xl" />
          <div className="flex-1 space-y-2">
            <Skeleton className="h-6 w-24" />
            <Skeleton className="h-4 w-full max-w-md" />
            <Skeleton className="h-5 w-20" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={statusCardClass(grade)}>
      <CardContent className="flex items-start gap-3 py-0 sm:items-center sm:gap-4">
        <div className="bg-background/40 flex size-10 shrink-0 items-center justify-center rounded-xl border border-border/50 shadow-sm sm:size-12">
          <Icon className="size-5 sm:size-6" />
        </div>
        <div className="min-w-0 space-y-1">
          <div className="flex flex-wrap items-center gap-1.5 sm:gap-2">
            <p className="text-xl font-bold tracking-wide sm:text-2xl">
              {grade ? grade.toUpperCase() : "—"}
            </p>
            {score != null && (
              <Badge variant={grade ? gradeBadgeVariant(grade) : "muted"}>
                Score {score}/100
              </Badge>
            )}
            {dnsIntegrity && (
              <Badge
                variant={integrityBadgeVariant(dnsIntegrity.state)}
                title={dnsIntegrity.summary}
              >
                DNS {integrityLabel(dnsIntegrity.state)}
              </Badge>
            )}
          </div>
          <p className="text-sm leading-relaxed">{summary}</p>
        </div>
      </CardContent>
    </Card>
  );
}
