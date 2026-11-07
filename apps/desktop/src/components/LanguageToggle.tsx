import { Button } from "@/components/ui/button";
import { useLocale } from "@/hooks/useLocale";
import { cn } from "@/lib/utils";

export function LanguageToggle({
  compact = false,
  className,
}: {
  compact?: boolean;
  className?: string;
}) {
  const { locale, setLocale, options, t } = useLocale();

  return (
    <div
      className={cn(
        "flex items-center gap-0.5 rounded-md border border-border/60 bg-card/50 p-0.5 shadow-sm backdrop-blur-sm",
        compact ? "w-full justify-between" : "w-fit flex-wrap",
        className,
      )}
      role="group"
      aria-label={t("language.group")}
    >
      {options.map((option) => {
        const active = locale === option.code;
        return (
          <Button
            key={option.code}
            type="button"
            size="sm"
            variant={active ? "default" : "secondary"}
            className={cn(
              "h-8 gap-1.5 rounded-lg px-3",
              compact && "flex-1",
              active && "hub-rail-active",
              !active && "text-muted-foreground hover:text-foreground",
            )}
            aria-pressed={active}
            title={option.name}
            onClick={() => setLocale(option.code)}
          >
            <span className="text-xs">{option.nativeName}</span>
          </Button>
        );
      })}
    </div>
  );
}
