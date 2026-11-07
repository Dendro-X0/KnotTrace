import type { HealthGrade, HistoryTrendPoint } from "../types.ts";

/** Quick Overview: latest exit only — avoid a peer “history deck” under the hero. */
export const PATH_EXIT_JOURNAL_QUICK_LIMIT = 1;
export const PATH_EXIT_JOURNAL_ADVANCED_LIMIT = 8;

export interface PathExitJournalRow {
  at: string;
  grade: HealthGrade;
  activePath: string | null;
  egressIp: string | null;
  slowdownShape: string | null;
  /** Stable plain line for tests / non-i18n fallback. */
  summaryLine: string;
}

export interface PathExitJournal {
  rows: PathExitJournalRow[];
  empty: boolean;
}

function hasJournalSignal(point: HistoryTrendPoint): boolean {
  return Boolean(
    point.grade ||
      point.active_path ||
      point.egress_ip ||
      point.slowdown_shape,
  );
}

function formatClock(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function buildSummaryLine(point: HistoryTrendPoint): string {
  const parts = [
    formatClock(point.timestamp),
    point.grade,
    point.active_path?.replace(/_/g, " ") ?? "—",
    point.egress_ip ?? "—",
    point.slowdown_shape?.replace(/_/g, " ") ?? "—",
  ];
  return parts.join(" · ");
}

/**
 * Compact path/exit journal from history trends (newest first).
 * Does not invent active_path when missing on legacy rows.
 */
export function buildPathExitJournal(
  points: HistoryTrendPoint[],
  limit: number = PATH_EXIT_JOURNAL_ADVANCED_LIMIT,
): PathExitJournal {
  const capped = Math.max(0, limit);
  const newestFirst = [...points].reverse();
  const rows: PathExitJournalRow[] = [];

  for (const point of newestFirst) {
    if (!hasJournalSignal(point)) continue;
    rows.push({
      at: point.timestamp,
      grade: point.grade,
      activePath: point.active_path ?? null,
      egressIp: point.egress_ip ?? null,
      slowdownShape: point.slowdown_shape ?? null,
      summaryLine: buildSummaryLine(point),
    });
    if (rows.length >= capped) break;
  }

  return { rows, empty: rows.length === 0 };
}
