import { Route } from "lucide-react";

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
}

export function QuickPathTeaser({ loading, report, tunnelCompare }: QuickPathTeaserProps) {
  const { t } = useLocale();

  if (loading) {
    return (
      <Card className="h-full border-border/50">
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
    <Card className="h-full border-border/50">
      <CardHeader className="pb-1.5">
        <div className="flex flex-wrap items-center gap-2">
          <Route className="text-muted-foreground size-3.5" />
          <CardTitle className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Path
          </CardTitle>
          {report?.active_path && (
            <Badge variant={content.tunnelActive ? "caution" : "info"}>
              {formatTag(report.active_path)}
            </Badge>
          )}
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
