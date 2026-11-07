import { useState } from "react";
import { FileJson } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { CompanionState } from "@/hooks/useCompanion";

interface DiagnosticBundlePanelProps {
  state: CompanionState;
  compact?: boolean;
  flush?: boolean;
}

export function DiagnosticBundlePanel({
  state,
  compact = false,
  flush = false,
}: DiagnosticBundlePanelProps) {
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const hasReport = Boolean(state.report);

  async function handleExport() {
    setBusy(true);
    setStatus(null);
    try {
      const text = await state.exportDiagnosticBundle(false);
      await navigator.clipboard.writeText(text);
      setStatus("Copied redacted diagnostic JSON. Paste into an AI only if you accept sharing these fields.");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : "Could not export diagnostic bundle.",
      );
    } finally {
      setBusy(false);
    }
  }

  const shell = flush
    ? "h-full rounded-none border-0 bg-transparent shadow-none"
    : compact
      ? "h-full border-border/50"
      : "border-border/70 bg-muted/10";

  if (compact) {
    return (
      <Card className={cn(shell, "justify-between gap-3")}>
        <CardHeader className="pb-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <FileJson className="text-muted-foreground size-3.5" />
            <CardTitle className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
              Export
            </CardTitle>
          </div>
        </CardHeader>
        <CardContent className="flex flex-1 flex-col gap-2">
          <p className="text-muted-foreground text-xs leading-relaxed">
            Redacted check facts for sharing. Nothing uploads until you paste elsewhere.
          </p>
          {!hasReport && (
            <p className="text-xs leading-relaxed" role="status">
              Export needs a completed report — run a health check first.
            </p>
          )}
          {status && (
            <p
              className={`text-xs leading-relaxed ${status.toLowerCase().includes("could not") || status.toLowerCase().includes("failed") || status.toLowerCase().includes("before exporting") ? "text-destructive" : ""}`}
              role="status"
            >
              {status}
            </p>
          )}
          <div className="mt-auto pt-1">
            {hasReport ? (
              <Button
                size="sm"
                variant="secondary"
                className="w-full"
                disabled={busy}
                onClick={() => void handleExport()}
              >
                {busy ? "Exporting…" : "Copy JSON"}
              </Button>
            ) : (
              <Button
                size="sm"
                className="w-full"
                disabled={state.checking}
                onClick={() => void state.runCheck()}
              >
                {state.checking ? "Checking…" : "Run health check"}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={shell}>
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <FileJson className="size-4" />
            </div>
            <CardTitle className="text-base">Diagnostic export</CardTitle>
          </div>
          {hasReport ? (
            <Button size="sm" variant="secondary" disabled={busy} onClick={() => void handleExport()}>
              {busy ? "Exporting…" : "Copy JSON"}
            </Button>
          ) : (
            <Button size="sm" disabled={state.checking} onClick={() => void state.runCheck()}>
              {state.checking ? "Checking…" : "Run health check"}
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-muted-foreground text-xs leading-relaxed">
          AI-ready facts from the latest check. Hostname and SSID stay out by default. No cloud
          upload — copy stays on this machine until you paste it elsewhere.
        </p>
        {!hasReport && (
          <p className="text-xs leading-relaxed" role="status">
            Export needs a completed report on this device — run a health check first.
          </p>
        )}
        {status && (
          <p
            className={`text-xs leading-relaxed ${status.toLowerCase().includes("could not") || status.toLowerCase().includes("failed") || status.toLowerCase().includes("before exporting") ? "text-destructive" : ""}`}
            role="status"
          >
            {status}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
