import { AlertTriangle, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";

interface CheckFailureBannerProps {
  message: string;
  checking?: boolean;
  onRetry: () => void;
}

export function CheckFailureBanner({ message, checking, onRetry }: CheckFailureBannerProps) {
  return (
    <div
      role="alert"
      className="border-destructive/40 bg-destructive/10 flex flex-wrap items-start justify-between gap-3 rounded-xl border px-3 py-2.5"
    >
      <div className="flex min-w-0 items-start gap-2">
        <AlertTriangle className="text-destructive mt-0.5 size-4 shrink-0" />
        <div className="min-w-0 space-y-1">
          <p className="text-destructive text-sm font-medium">Health check did not finish</p>
          <p className="text-destructive/90 text-xs leading-relaxed">{message}</p>
          <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
            Try again. If it keeps failing, note your OS and app version, then open a GitHub issue
            (optional: export a diagnostic JSON after a successful check).
          </p>
        </div>
      </div>
      <Button size="sm" disabled={checking} onClick={onRetry}>
        <RefreshCw className="size-4" />
        {checking ? "Retrying…" : "Retry check"}
      </Button>
    </div>
  );
}
