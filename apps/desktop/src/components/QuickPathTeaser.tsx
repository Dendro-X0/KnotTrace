import { Route } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { PathBreakReport } from "@/types";
import { formatTag } from "@/types";

interface QuickPathTeaserProps {
  loading?: boolean;
  report?: PathBreakReport | null;
}

export function QuickPathTeaser({ loading, report }: QuickPathTeaserProps) {
  if (loading) {
    return (
      <Card>
        <CardContent className="py-3">
          <Skeleton className="h-14 w-full rounded-lg" />
        </CardContent>
      </Card>
    );
  }

  const title = report?.title ?? "Path not classified yet";
  const summary =
    report?.summary ?? "Run a health check to see whether the issue looks local or on the path.";

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center gap-2">
          <Route className="text-muted-foreground size-4" />
          <CardTitle className="text-sm">Path</CardTitle>
          {report?.active_path && (
            <Badge variant="info">{formatTag(report.active_path)}</Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-1">
        <p className="text-sm font-medium leading-snug">{title}</p>
        <p className="text-muted-foreground text-xs leading-relaxed">{summary}</p>
      </CardContent>
    </Card>
  );
}
