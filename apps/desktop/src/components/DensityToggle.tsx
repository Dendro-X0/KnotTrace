import { Gauge, Layers } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useLocale } from "@/hooks/useLocale";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { UiDensityMode } from "@/lib/uiDensity";
import { cn } from "@/lib/utils";

const OPTIONS: Array<{ mode: UiDensityMode; labelKey: string; icon: typeof Gauge }> = [
  { mode: "quick", labelKey: "density.quick", icon: Gauge },
  { mode: "advanced", labelKey: "density.advanced", icon: Layers },
];

export function DensityToggle({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  const { mode, setMode } = useUiDensity();
  const { t } = useLocale();

  return (
    <div
      className={cn(
        "flex items-center gap-0.5 rounded-md border border-border/60 bg-card/50 p-0.5 shadow-sm backdrop-blur-sm",
        compact ? "w-full justify-between" : "w-fit",
        className,
      )}
      role="group"
      aria-label={t("density.group")}
    >
      {OPTIONS.map(({ mode: option, labelKey, icon: Icon }) => {
        const active = mode === option;
        return (
          <Button
            key={option}
            type="button"
            size="sm"
            variant={active ? "default" : "secondary"}
            className={cn(
              "h-8 gap-1.5 rounded-lg px-2.5",
              compact && "flex-1",
              active && "hub-rail-active",
              !active && "text-muted-foreground hover:text-foreground",
            )}
            aria-pressed={active}
            onClick={() => setMode(option)}
          >
            <Icon className="size-3.5" />
            <span className={cn("text-xs", compact ? "inline" : "hidden sm:inline")}>
              {t(labelKey)}
            </span>
          </Button>
        );
      })}
    </div>
  );
}
