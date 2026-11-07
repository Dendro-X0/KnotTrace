import { cn } from "@/lib/utils";

export interface MetricStripItem {
  label: string;
  value: string;
}

interface MetricStripProps {
  metrics: MetricStripItem[];
  className?: string;
}

/** Equal metric chips — shared by Overview hero and Network Quick link snapshot. */
export function MetricStrip({ metrics, className }: MetricStripProps) {
  if (metrics.length === 0) return null;

  return (
    <div
      className={cn(
        "mx-auto grid w-full max-w-lg gap-2",
        metrics.length === 1 && "grid-cols-1",
        metrics.length === 2 && "grid-cols-2",
        metrics.length >= 3 && "grid-cols-3",
        className,
      )}
    >
      {metrics.map((metric) => (
        <div
          key={metric.label}
          className="rounded-xl border border-border/45 bg-background/40 px-3 py-2.5 text-center shadow-sm backdrop-blur-sm"
        >
          <p className="text-muted-foreground text-[0.62rem] font-medium tracking-wide uppercase">
            {metric.label}
          </p>
          <p className="mt-0.5 text-sm font-semibold tabular-nums sm:text-base">{metric.value}</p>
        </div>
      ))}
    </div>
  );
}
