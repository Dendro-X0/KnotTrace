import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
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
    <Card className="border-border/50 bg-muted/15">
      <CardContent className="flex flex-col gap-2 py-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
        <p className="text-sm leading-relaxed">{t("overviewTip.pathTruth")}</p>
        <Button
          size="sm"
          variant="secondary"
          className="shrink-0 self-start sm:self-auto"
          onClick={() => {
            dismissPathTruthTip();
            setVisible(false);
          }}
        >
          {t("overviewTip.dismiss")}
        </Button>
      </CardContent>
    </Card>
  );
}
