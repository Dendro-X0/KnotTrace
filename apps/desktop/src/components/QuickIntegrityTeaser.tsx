import { ShieldAlert } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import type { IntegrityBoardReport } from "@/types";

interface QuickIntegrityTeaserProps {
  loading?: boolean;
  report?: IntegrityBoardReport | null;
  checking?: boolean;
  onRunCheck?: () => void;
  onOpenProtect?: () => void;
  flush?: boolean;
}

function levelVariant(level: IntegrityBoardReport["level"] | undefined) {
  if (level === "elevated" || level === "caution") return "caution" as const;
  if (level === "info") return "info" as const;
  return "muted" as const;
}

export function QuickIntegrityTeaser({
  loading,
  report,
  checking,
  onRunCheck,
  onOpenProtect,
  flush = false,
}: QuickIntegrityTeaserProps) {
  const shell = flush
    ? "h-full rounded-none border-0 bg-transparent shadow-none"
    : "h-full border-border/50";

  if (loading) {
    return (
      <Card className={shell}>
        <CardContent className="py-3">
          <Skeleton className="h-14 w-full rounded-lg" />
        </CardContent>
      </Card>
    );
  }

  if (!report) {
    return (
      <Card className={shell}>
        <CardHeader className="pb-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <ShieldAlert className="text-muted-foreground size-3.5" />
            <CardTitle className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
              Integrity
            </CardTitle>
            <Badge variant="muted" className="normal-case">
              Needs a check
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-2">
          <p className="text-muted-foreground text-xs leading-relaxed">
            Run a health check to load exposure indicators. Empty is not “secure.”
          </p>
          {onRunCheck && (
            <Button size="sm" disabled={checking} onClick={onRunCheck}>
              {checking ? "Checking…" : "Run health check"}
            </Button>
          )}
        </CardContent>
      </Card>
    );
  }

  const count = report.indicator_count;
  const level = report.level;
  const openable = Boolean(onOpenProtect) && count > 0;

  return (
    <Card
      className={cn(shell, openable && "cursor-pointer")}
      onClick={openable ? onOpenProtect : undefined}
      role={openable ? "button" : undefined}
      tabIndex={openable ? 0 : undefined}
      onKeyDown={
        openable
          ? (event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onOpenProtect?.();
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
            <Badge variant={levelVariant(level)} className="normal-case">
              {count === 1 ? "1 signal" : `${count} signals`}
            </Badge>
          ) : (
            <Badge variant="muted" className="normal-case">
              Clear this check
            </Badge>
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
