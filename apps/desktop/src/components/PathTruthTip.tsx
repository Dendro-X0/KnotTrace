import { useState } from "react";

import { Button } from "@/components/ui/button";
import { useLocale } from "@/hooks/useLocale";
import {
  dismissPathTruthTip,
  readPathTruthTipDismissed,
  shouldShowPathTruthTip,
} from "@/lib/pathTruthTip";

export function PathTruthTip() {
  const { t } = useLocale();
  const [visible, setVisible] = useState(() =>
    shouldShowPathTruthTip(readPathTruthTipDismissed()),
  );

  if (!visible) return null;

  return (
    <div className="border-border/30 flex flex-col gap-2 border-b pb-2 sm:flex-row sm:items-center sm:justify-between sm:gap-3">
      <p className="text-muted-foreground text-xs leading-snug sm:text-[0.8rem]">
        {t("overviewTip.pathTruth")}
      </p>
      <Button
        size="sm"
        variant="secondary"
        className="text-muted-foreground h-7 shrink-0 self-start px-2 text-xs sm:self-auto"
        onClick={() => {
          dismissPathTruthTip();
          setVisible(false);
        }}
      >
        {t("overviewTip.dismiss")}
      </Button>
    </div>
  );
}
