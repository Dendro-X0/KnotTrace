import { useState } from "react";
import { FileJson } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { CompanionState } from "@/hooks/useCompanion";

interface DiagnosticBundlePanelProps {
  state: CompanionState;
  compact?: boolean;
}

export function DiagnosticBundlePanel({ state, compact = false }: DiagnosticBundlePanelProps) {
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const disabled = busy || !state.report;

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

  return (
    <Card
      className={cn(
        "border-border/60",
        compact ? "h-full justify-between gap-3 py-3.5" : "border-border/70 bg-muted/10",
      )}
    >
      <CardHeader className={cn("pb-2", compact && "items-start")}>
        <div className={cn("flex gap-2", compact ? "flex-col" : "flex-wrap items-center justify-between")}>
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <FileJson className="size-4" />
            </div>
            <CardTitle className={compact ? "text-sm" : "text-base"}>Diagnostic export</CardTitle>
          </div>
          <Button
            size="sm"
            variant="secondary"
            className={compact ? "w-full" : undefined}
            disabled={disabled}
            onClick={() => void handleExport()}
          >
            {busy ? "Exporting…" : "Copy JSON"}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p
          className={cn(
            "text-muted-foreground text-xs leading-relaxed",
            compact && "line-clamp-4",
          )}
        >
          {compact
            ? "Redacted check facts for sharing. Nothing uploads until you paste elsewhere."
            : "AI-ready facts from the latest check. Hostname and SSID stay out by default. No cloud upload — copy stays on this machine until you paste it elsewhere."}
        </p>
        {!state.report && (
          <p className="text-xs leading-relaxed" role="status">
            Run a health check first — export needs a completed report on this device.
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
