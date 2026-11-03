import { Monitor, Moon, Sun } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useLocale } from "@/hooks/useLocale";
import { useTheme } from "@/hooks/useTheme";
import type { ThemeMode } from "@/lib/theme";
import { cn } from "@/lib/utils";

const OPTIONS: Array<{ mode: ThemeMode; labelKey: string; icon: typeof Sun }> = [
  { mode: "light", labelKey: "theme.light", icon: Sun },
  { mode: "dark", labelKey: "theme.dark", icon: Moon },
  { mode: "system", labelKey: "theme.system", icon: Monitor },
];

export function ThemeToggle({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  const { mode, setMode } = useTheme();
  const { t } = useLocale();

  return (
    <div
      className={cn(
        "flex items-center gap-0.5 rounded-md border border-border/60 bg-card/50 p-0.5 shadow-sm backdrop-blur-sm",
        compact ? "w-full justify-between" : "w-fit",
        className,
      )}
      role="group"
      aria-label={t("theme.group")}
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
              "h-7 gap-1.5 rounded-[0.3rem] px-2",
              compact && "flex-1",
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
