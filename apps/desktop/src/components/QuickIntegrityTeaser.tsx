import { ShieldAlert } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { IntegrityBoardReport } from "@/types";

interface QuickIntegrityTeaserProps {
  loading?: boolean;
  report?: IntegrityBoardReport | null;
  onOpenProtect: () => void;
}

function levelVariant(level: IntegrityBoardReport["level"] | undefined) {
  if (level === "elevated" || level === "caution") return "caution" as const;
  if (level === "info") return "info" as const;
  return "muted" as const;
}

export function QuickIntegrityTeaser({
  loading,
  report,
  onOpenProtect,
}: QuickIntegrityTeaserProps) {
  if (loading) {
    return (
      <Card>
        <CardContent className="py-3">
          <Skeleton className="h-16 w-full rounded-lg" />
        </CardContent>
      </Card>
    );
  }

  const count = report?.indicator_count ?? 0;
  const level = report?.level ?? "none";

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center gap-2">
          <ShieldAlert className="text-muted-foreground size-4" />
          <CardTitle className="text-sm">Integrity</CardTitle>
          {count > 0 ? (
            <Badge variant={levelVariant(level)}>{count} indicator(s)</Badge>
          ) : (
            <Badge variant="muted">No indicators this check</Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-muted-foreground text-xs leading-relaxed">
          {count > 0
            ? "Exposure or routing indicators were found. Review them on Protect — these are not a malware verdict."
            : "Absence of indicators does not mean you are unobserved. Open Protect for opt-in scans."}
        </p>
        <Button size="sm" variant="secondary" onClick={onOpenProtect}>
          Open Protect
        </Button>
      </CardContent>
    </Card>
  );
}
