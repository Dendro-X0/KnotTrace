import { Route } from "lucide-react";

import { EnglishProbeNote } from "@/components/EnglishProbeNote";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { useLocale } from "@/hooks/useLocale";
import { buildQuickPathTeaserContent } from "@/lib/pathTeaser";
import type { PathBreakReport, TunnelPathCompareReport } from "@/types";
import { formatTag } from "@/types";

interface QuickPathTeaserProps {
  loading?: boolean;
  report?: PathBreakReport | null;
  tunnelCompare?: TunnelPathCompareReport | null;
  /** Borderless cell inside the command deck. */
  flush?: boolean;
}

export function QuickPathTeaser({
  loading,
  report,
  tunnelCompare,
  flush = false,
}: QuickPathTeaserProps) {
  const { t } = useLocale();
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

  const content = buildQuickPathTeaserContent(report, tunnelCompare);
  const expectationText =
    content.expectation == null
      ? null
      : content.expectation.kind === "key"
        ? t(content.expectation.key)
        : content.expectation.text;

  return (
    <Card className={shell}>
      <CardHeader className="pb-1.5">
        <div className="flex flex-wrap items-center gap-2">
          <Route className="text-muted-foreground size-3.5" />
          <CardTitle className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Path
          </CardTitle>
          {report?.active_path && (
            <Badge
              variant={content.tunnelActive ? "caution" : "info"}
              className="normal-case"
            >
              {formatTag(report.active_path)}
            </Badge>
          )}
          {report ? <EnglishProbeNote /> : null}
        </div>
      </CardHeader>
      <CardContent className="space-y-1">
        <p className="text-sm font-medium leading-snug">{content.title}</p>
        <p className="text-muted-foreground text-xs leading-relaxed xl:line-clamp-none line-clamp-3">
          {content.summary}
        </p>
        {expectationText && (
          <p className="text-foreground/80 border-border/50 mt-1 border-t pt-1.5 text-xs leading-relaxed">
            {expectationText}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
