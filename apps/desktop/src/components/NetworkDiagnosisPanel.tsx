import { Stethoscope } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { NetworkDiagnosis } from "@/types";

interface NetworkDiagnosisPanelProps {
  diagnosis?: NetworkDiagnosis | null;
}

function severityVariant(severity: "info" | "warning" | "critical") {
  if (severity === "critical") return "untrusted" as const;
  if (severity === "warning") return "caution" as const;
  return "info" as const;
}

function confidenceVariant(confidence: "high" | "medium" | "low") {
  if (confidence === "high") return "active" as const;
  if (confidence === "medium") return "caution" as const;
  return "info" as const;
}

function shapeLabel(shape: NetworkDiagnosis["slowdown_shape"]) {
  return shape.replace(/_/g, " ");
}

export function NetworkDiagnosisPanel({ diagnosis }: NetworkDiagnosisPanelProps) {
  if (!diagnosis) {
    return (
      <EmptyState
        icon={Stethoscope}
        title="Diagnosis pending"
        description="Run a health check to analyze likely bottlenecks on your current path."
      />
    );
  }

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-base">Network diagnosis</CardTitle>
          <div className="flex flex-wrap items-center gap-1">
            <Badge variant="info">{shapeLabel(diagnosis.slowdown_shape)}</Badge>
            <Badge variant={confidenceVariant(diagnosis.confidence)}>
              {diagnosis.confidence} confidence
            </Badge>
            {diagnosis.primary_bottleneck && diagnosis.primary_bottleneck !== "healthy" && (
              <Badge variant="caution">{diagnosis.primary_bottleneck.replace(/_/g, " ")}</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <p className="text-sm leading-relaxed">{diagnosis.summary}</p>
        <ScrollArea className="max-h-[min(18rem,42vh)]">
          <ul className="grid gap-2">
            {diagnosis.hints.map((hint) => (
              <li
                key={`${hint.category}-${hint.title}`}
                className="rounded-lg border border-border/70 bg-background/40 p-2.5 text-sm"
              >
                <div className="mb-1 flex items-center gap-2">
                  <Badge variant={severityVariant(hint.severity)}>{hint.severity}</Badge>
                  <span className="font-medium">{hint.title}</span>
                </div>
                <p className="text-muted-foreground text-xs leading-relaxed">{hint.message}</p>
                {hint.suggestions.length > 0 && (
                  <ul className="text-muted-foreground mt-2 list-disc space-y-1 pl-4 text-xs">
                    {hint.suggestions.map((suggestion) => (
                      <li key={suggestion}>{suggestion}</li>
                    ))}
                  </ul>
                )}
              </li>
            ))}
          </ul>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
