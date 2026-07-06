import type { ComponentType } from "react";

import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface MetricCardProps {
  label: string;
  value: string;
  icon: ComponentType<{ className?: string }>;
  tone?: "default" | "sky" | "emerald" | "amber";
}

const toneClasses = {
  default: "bg-primary/10 text-primary",
  sky: "bg-sky-500/10 text-sky-600 dark:text-sky-300",
  emerald: "bg-emerald-500/10 text-emerald-600 dark:text-emerald-300",
  amber: "bg-amber-500/10 text-amber-600 dark:text-amber-300",
};

export function MetricCard({ label, value, icon: Icon, tone = "default" }: MetricCardProps) {
  return (
    <Card className="bg-card/80 py-3 shadow-sm">
      <CardContent className="flex items-start justify-between gap-3 py-0">
        <div className="space-y-1">
          <p className="text-muted-foreground text-[0.68rem] uppercase tracking-wide">{label}</p>
          <p className="text-base font-semibold">{value}</p>
        </div>
        <div className={cn("flex size-8 shrink-0 items-center justify-center rounded-lg", toneClasses[tone])}>
          <Icon className="size-4" />
        </div>
      </CardContent>
    </Card>
  );
}
