import { NotebookTabs } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { gradeBadgeVariant } from "@/components/shared";
import { useLocale } from "@/hooks/useLocale";
import {
  PATH_EXIT_JOURNAL_ADVANCED_LIMIT,
  PATH_EXIT_JOURNAL_QUICK_LIMIT,
  buildPathExitJournal,
} from "@/lib/pathExitJournal";
import type { HistoryTrendPoint } from "@/types";
import { formatTag } from "@/types";

interface PathExitJournalProps {
  loading?: boolean;
  points: HistoryTrendPoint[];
  compact?: boolean;
  flush?: boolean;
}

export function PathExitJournal({
  loading,
  points,
  compact = false,
  flush = false,
}: PathExitJournalProps) {
  const { t } = useLocale();
  const limit = compact ? PATH_EXIT_JOURNAL_QUICK_LIMIT : PATH_EXIT_JOURNAL_ADVANCED_LIMIT;
  const journal = buildPathExitJournal(points, limit);

  if (loading) {
    return <Skeleton className={compact ? "h-12 rounded-lg" : "h-40 rounded-lg"} />;
  }

  if (compact && !flush) {
    const row = journal.rows[0];
    return (
      <div className="border-border/35 text-muted-foreground flex flex-wrap items-center gap-x-2 gap-y-1 border-t pt-2 text-xs">
        <NotebookTabs className="size-3.5 shrink-0 opacity-70" aria-hidden />
        <span className="font-medium tracking-wide uppercase">{t("pathJournal.latest")}</span>
        {journal.empty || !row ? (
          <span>{t("pathJournal.empty")}</span>
        ) : (
          <>
            <Badge variant={gradeBadgeVariant(row.grade)} className="normal-case">
              {row.grade}
            </Badge>
            <span>
              {new Date(row.at).toLocaleString(undefined, {
                month: "short",
                day: "numeric",
                hour: "2-digit",
                minute: "2-digit",
              })}
            </span>
            <span className="text-foreground/85">
              {row.activePath ? formatTag(row.activePath) : "—"}
            </span>
            <span className="font-mono">{row.egressIp ?? "—"}</span>
            <span>{row.slowdownShape ? formatTag(row.slowdownShape) : "—"}</span>
          </>
        )}
      </div>
    );
  }

  return (
    <Card
      className={
        flush
          ? "rounded-none border-0 bg-transparent shadow-none"
          : "border-border/70 bg-muted/10"
      }
    >
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center gap-2">
          <NotebookTabs className="text-muted-foreground size-3.5" />
          <CardTitle className="text-base">{t("pathJournal.title")}</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="space-y-1.5">
        {journal.empty ? (
          <p className="text-muted-foreground text-xs leading-relaxed">
            {t("pathJournal.empty")}
          </p>
        ) : (
          <ul className="space-y-1.5">
            {journal.rows.map((row) => (
              <li
                key={`${row.at}-${row.grade}-${row.activePath ?? ""}-${row.egressIp ?? ""}`}
                className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs"
              >
                <Badge variant={gradeBadgeVariant(row.grade)} className="normal-case">
                  {row.grade}
                </Badge>
                <span className="text-muted-foreground">
                  {new Date(row.at).toLocaleString(undefined, {
                    month: "short",
                    day: "numeric",
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </span>
                <span className="text-foreground/90">
                  {row.activePath ? formatTag(row.activePath) : "—"}
                </span>
                <span className="text-muted-foreground font-mono">
                  {row.egressIp ?? "—"}
                </span>
                <span className="text-muted-foreground">
                  {row.slowdownShape ? formatTag(row.slowdownShape) : "—"}
                </span>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
