import { useState } from "react";
import { FileJson } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { CompanionState } from "@/hooks/useCompanion";

interface DiagnosticBundlePanelProps {
  state: CompanionState;
}

export function DiagnosticBundlePanel({ state }: DiagnosticBundlePanelProps) {
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
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <FileJson className="size-4" />
            </div>
            <CardTitle className="text-base">Diagnostic export</CardTitle>
          </div>
          <Button size="sm" variant="secondary" disabled={disabled} onClick={() => void handleExport()}>
            {busy ? "Exporting…" : "Copy JSON"}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-muted-foreground text-xs leading-relaxed">
          AI-ready facts from the latest check. Hostname and SSID stay out by default. No cloud upload —
          copy stays on this machine until you paste it elsewhere.
        </p>
        {status && <p className="text-xs leading-relaxed">{status}</p>}
      </CardContent>
    </Card>
  );
}
