import { Badge } from "@/components/ui/badge";
import { useLocale } from "@/hooks/useLocale";
import {
  ENGLISH_PROBE_LABEL_KEY,
  needsEnglishProbeLabel,
} from "@/lib/probeLanguage";

interface EnglishProbeNoteProps {
  /** `badge` for panel headers; `note` for hero strips. */
  variant?: "badge" | "note";
  className?: string;
}

export function EnglishProbeNote({
  variant = "badge",
  className,
}: EnglishProbeNoteProps) {
  const { locale, t } = useLocale();
  if (!needsEnglishProbeLabel(locale)) return null;

  const label = t(ENGLISH_PROBE_LABEL_KEY);
  if (variant === "note") {
    return (
      <p className={className ?? "text-muted-foreground text-[0.7rem] leading-relaxed"}>
        {label}
      </p>
    );
  }

  return (
    <Badge variant="muted" className={className}>
      {label}
    </Badge>
  );
}
