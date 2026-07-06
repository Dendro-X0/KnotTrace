import { cn } from "@/lib/utils";
import type { HealthGrade } from "@/types";

interface FactListProps {
  rows: Array<{ label: string; value: string }>;
  className?: string;
}

export function FactList({ rows, className }: FactListProps) {
  return (
    <dl className={cn("grid gap-2", className)}>
      {rows.map((row) => (
        <div key={row.label} className="grid grid-cols-[110px_1fr] gap-2 text-sm">
          <dt className="text-muted-foreground">{row.label}</dt>
          <dd className="break-words">{row.value}</dd>
        </div>
      ))}
    </dl>
  );
}

export function gradeBadgeVariant(grade: HealthGrade) {
  if (grade === "good") return "good" as const;
  if (grade === "fair") return "fair" as const;
  return "poor" as const;
}

export function statusCardClass(grade?: HealthGrade) {
  if (grade === "good") return "border-emerald-500/30 bg-gradient-to-br from-emerald-500/12 to-card/80";
  if (grade === "fair") return "border-amber-500/30 bg-gradient-to-br from-amber-500/12 to-card/80";
  if (grade === "poor") return "border-rose-500/30 bg-gradient-to-br from-rose-500/12 to-card/80";
  return "border-border/70 bg-card/70";
}

export function formatLatency(value: number | null | undefined, fallback = "Unavailable") {
  return value == null ? fallback : `${value.toFixed(0)} ms`;
}

export function integrityBadgeVariant(state: "ok" | "caution" | "suspicious" | undefined) {
  if (state === "suspicious") return "untrusted" as const;
  if (state === "caution") return "caution" as const;
  if (state === "ok") return "active" as const;
  return "muted" as const;
}

export function integrityLabel(state: "ok" | "caution" | "suspicious" | undefined) {
  if (state === "suspicious") return "Suspicious";
  if (state === "caution") return "Caution";
  if (state === "ok") return "OK";
  return "Unknown";
}
