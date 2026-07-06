import { LineChart } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import type { HistoryTrendPoint } from "@/types";

function buildPath(
  values: number[],
  width: number,
  height: number,
  maxValue: number,
): string {
  if (values.length === 0) return "";

  const stepX = values.length > 1 ? width / (values.length - 1) : 0;

  return values
    .map((value, index) => {
      const x = index * stepX;
      const y = height - (value / maxValue) * height;
      return `${index === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function integrityTrendValue(point: HistoryTrendPoint): number | null {
  if (point.dns_integrity_mismatch_count != null) {
    return point.dns_integrity_mismatch_count;
  }
  if (point.dns_integrity_state === "ok") return 0;
  if (point.dns_integrity_state === "caution") return 1;
  if (point.dns_integrity_state === "suspicious") return 2;
  return null;
}

function integrityTrendLabel(value: number) {
  if (value <= 0) return "OK";
  if (value === 1) return "1 mismatch";
  return `${value} mismatches`;
}

interface MiniChartProps {
  title: string;
  values: Array<number | null>;
  color: string;
  maxValue: number;
  unit: string;
  formatLatest?: (value: number) => string;
}

function MiniChart({ title, values, color, maxValue, unit, formatLatest }: MiniChartProps) {
  const numeric = values.filter((value): value is number => value != null);

  if (numeric.length < 2) {
    return (
      <div className="rounded-lg border border-border/70 bg-muted/30 p-3">
        <div className="mb-1 flex items-center justify-between text-xs text-muted-foreground">
          <span>{title}</span>
          <span>—</span>
        </div>
        <p className="text-muted-foreground text-xs">Not enough data yet.</p>
      </div>
    );
  }

  const width = 320;
  const height = 48;
  const latest = numeric[numeric.length - 1];
  const path = buildPath(numeric, width, height, maxValue);
  const latestLabel = formatLatest ? formatLatest(latest) : `${latest.toFixed(0)}${unit}`;

  return (
    <div className="rounded-lg border border-border/70 bg-muted/30 p-3">
      <div className="mb-1 flex items-center justify-between text-xs text-muted-foreground">
        <span>{title}</span>
        <span>{latestLabel}</span>
      </div>
      <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" className="h-12 w-full" aria-hidden>
        <path d={path} fill="none" stroke={color} strokeWidth="2.5" strokeLinecap="round" />
      </svg>
    </div>
  );
}

interface TrendChartsProps {
  points: HistoryTrendPoint[];
  error?: string | null;
}

export function TrendCharts({ points, error }: TrendChartsProps) {
  if (error) {
    return (
      <EmptyState
        icon={LineChart}
        title="Trend data unavailable"
        description={error}
        className="py-8"
      />
    );
  }

  if (points.length < 2) {
    return (
      <EmptyState
        icon={LineChart}
        title="Not enough trend data"
        description="Run a few health checks to see score and latency trends over time."
        className="py-8"
      />
    );
  }

  const scores = points.map((point) => point.score);
  const dns = points.map((point) => point.dns_latency_ms);
  const internet = points.map((point) => point.internet_latency_ms);
  const integrity = points.map(integrityTrendValue);

  const dnsMax = Math.max(50, ...dns.filter((value): value is number => value != null));
  const internetMax = Math.max(
    20,
    ...internet.filter((value): value is number => value != null),
  );
  const integrityMax = Math.max(
    1,
    ...integrity.filter((value): value is number => value != null),
  );
  const hasIntegrityData = integrity.some((value) => value != null);

  return (
    <div className="grid grid-cols-1 gap-3 lg:grid-cols-2 xl:grid-cols-4">
      <MiniChart title="Health score" values={scores} color="#3b82f6" maxValue={100} unit="/100" />
      <MiniChart title="DNS latency" values={dns} color="#22c55e" maxValue={dnsMax * 1.2} unit=" ms" />
      <MiniChart
        title="Internet latency"
        values={internet}
        color="#f59e0b"
        maxValue={internetMax * 1.2}
        unit=" ms"
      />
      {hasIntegrityData ? (
        <MiniChart
          title="DNS integrity"
          values={integrity}
          color="#a855f7"
          maxValue={Math.max(integrityMax, 1) * 1.2}
          unit=""
          formatLatest={integrityTrendLabel}
        />
      ) : (
        <div className="rounded-lg border border-border/70 bg-muted/30 p-3">
          <div className="mb-1 flex items-center justify-between text-xs text-muted-foreground">
            <span>DNS integrity</span>
            <span>—</span>
          </div>
          <p className="text-muted-foreground text-xs">
            Run more health checks to build integrity trend data.
          </p>
        </div>
      )}
    </div>
  );
}
