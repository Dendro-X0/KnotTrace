import { Monitor, Moon, Sun } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useTheme } from "@/hooks/useTheme";
import type { ThemeMode } from "@/lib/theme";
import { cn } from "@/lib/utils";

const OPTIONS: Array<{ mode: ThemeMode; label: string; icon: typeof Sun }> = [
  { mode: "light", label: "Light", icon: Sun },
  { mode: "dark", label: "Dark", icon: Moon },
  { mode: "system", label: "System", icon: Monitor },
];

export function ThemeToggle({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  const { mode, setMode } = useTheme();

  return (
    <div
      className={cn(
        "flex items-center gap-0.5 rounded-md border border-border/60 bg-card/50 p-0.5 shadow-sm backdrop-blur-sm",
        compact ? "w-full justify-between" : "w-fit",
        className,
      )}
      role="group"
      aria-label="Theme"
    >
      {OPTIONS.map(({ mode: option, label, icon: Icon }) => {
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
            <span className={cn("text-xs", compact ? "inline" : "hidden sm:inline")}>{label}</span>
          </Button>
        );
      })}
    </div>
  );
}
