import { ShieldAlert } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import type { IntegrityBoardReport } from "@/types";

interface QuickIntegrityTeaserProps {
  loading?: boolean;
  report?: IntegrityBoardReport | null;
  onOpenProtect?: () => void;
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
      <Card className="h-full border-border/50">
        <CardContent className="py-3">
          <Skeleton className="h-14 w-full rounded-lg" />
        </CardContent>
      </Card>
    );
  }

  const count = report?.indicator_count ?? 0;
  const level = report?.level ?? "none";

  return (
    <Card
      className={cn(
        "h-full border-border/50",
        onOpenProtect && "cursor-pointer",
      )}
      onClick={onOpenProtect}
      role={onOpenProtect ? "button" : undefined}
      tabIndex={onOpenProtect ? 0 : undefined}
      onKeyDown={
        onOpenProtect
          ? (event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onOpenProtect();
              }
            }
          : undefined
      }
    >
      <CardHeader className="pb-1.5">
        <div className="flex flex-wrap items-center gap-2">
          <ShieldAlert className="text-muted-foreground size-3.5" />
          <CardTitle className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Integrity
          </CardTitle>
          {count > 0 ? (
            <Badge variant={levelVariant(level)}>{count} indicator(s)</Badge>
          ) : (
            <Badge variant="muted">Clear this check</Badge>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground text-xs leading-relaxed">
          {count > 0
            ? "Exposure or routing signals found — open Protect to review. Not a malware verdict."
            : "No indicators on this check. Absence does not mean you are unobserved."}
        </p>
      </CardContent>
    </Card>
  );
}
