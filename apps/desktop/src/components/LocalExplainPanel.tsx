import { useState } from "react";
import { Sparkles } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { CompanionState } from "@/hooks/useCompanion";
import type { LocalExplainReport, PageId } from "@/types";

interface LocalExplainPanelProps {
  state: CompanionState;
}

function pageForAction(actionId: string): PageId | null {
  switch (actionId) {
    case "open_dns_assist":
      return "dns";
    case "open_connect":
      return "connect";
    case "open_protect":
      return "protect";
    case "open_network":
    case "review_mtu":
    case "review_local_caps":
      return "network";
    default:
      return null;
  }
}

export function LocalExplainPanel({ state }: LocalExplainPanelProps) {
  const [explain, setExplain] = useState<LocalExplainReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const disabled = busy || !state.report;

  async function handleExplain() {
    setBusy(true);
    setError(null);
    try {
      const result = await state.explainLocally();
      setExplain(result);
    } catch (err) {
      setExplain(null);
      setError(err instanceof Error ? err.message : "Could not build local explanation.");
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
              <Sparkles className="size-4" />
            </div>
            <CardTitle className="text-base">Explain locally</CardTitle>
          </div>
          <Button size="sm" variant="secondary" disabled={disabled} onClick={() => void handleExplain()}>
            {busy ? "Reading…" : explain ? "Refresh" : "Explain"}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-muted-foreground text-xs leading-relaxed">
          Plain-language reading of this check’s measurements on-device. No cloud, no invented probes —
          next steps only navigate; repairs still need your confirm.
        </p>
        {error && <p className="text-xs text-destructive">{error}</p>}
        {explain && (
          <div className="space-y-2 rounded-lg border border-border/70 bg-background/40 p-3">
            <p className="text-sm font-medium leading-relaxed">{explain.headline}</p>
            <p className="text-muted-foreground text-xs leading-relaxed">{explain.plain_summary}</p>
            {explain.likely_cause && (
              <p className="text-xs leading-relaxed">
                <span className="font-medium">Likely cause: </span>
                {explain.likely_cause}
              </p>
            )}
            {explain.next_steps.length > 0 && (
              <div className="flex flex-wrap gap-2 pt-1">
                {explain.next_steps.map((step) => {
                  const page = pageForAction(step.action_id);
                  return (
                    <Button
                      key={`${step.action_id}-${step.label}`}
                      size="sm"
                      variant="secondary"
                      title={step.reason}
                      disabled={!page}
                      onClick={() => page && state.navigate(page)}
                    >
                      {step.label}
                    </Button>
                  );
                })}
              </div>
            )}
            {explain.caveats.length > 0 && (
              <ul className="text-muted-foreground list-disc space-y-1 pl-4 text-xs">
                {explain.caveats.slice(0, 4).map((caveat) => (
                  <li key={caveat}>{caveat}</li>
                ))}
              </ul>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
