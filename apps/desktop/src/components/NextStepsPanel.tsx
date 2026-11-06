import { ArrowRight, Gauge, ShieldAlert, ShieldCheck, Workflow } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { CompanionState } from "@/hooks/useCompanion";
import { useLocale } from "@/hooks/useLocale";
import { buildNextStepPlan, extractNextStepInput } from "@/lib/nextStepPlan";

interface NextStepsPanelProps {
  state: CompanionState;
}

export { buildNextStepPlan } from "@/lib/nextStepPlan";

export function NextStepsPanel({ state }: NextStepsPanelProps) {
  const { t } = useLocale();
  const loading = state.bootstrapping && !state.report;
  const plan = buildNextStepPlan(state);
  const diagnosis = state.report?.diagnosis;
  const input = extractNextStepInput(state);
  const recurrence =
    input && input.recurrenceCount >= 2
      ? { shape: input.shape, count: input.recurrenceCount }
      : null;

  if (loading) {
    return (
      <Card className="border-border/70 bg-card/80">
        <CardHeader className="pb-2">
          <CardTitle>{t("nextSteps.title")}</CardTitle>
        </CardHeader>
        <CardContent className="text-muted-foreground text-sm">{t("nextSteps.analyzing")}</CardContent>
      </Card>
    );
  }

  if (!plan || !diagnosis) {
    return (
      <EmptyState
        icon={Workflow}
        title={t("nextSteps.pendingTitle")}
        description={t("nextSteps.pendingDescription")}
        action={
          <Button size="sm" disabled={state.checking} onClick={() => void state.runCheck()}>
            {state.checking ? t("header.checking") : t("header.runCheck")}
          </Button>
        }
      />
    );
  }

  return (
    <Card className="border-border/70 bg-card/80">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle>{t("nextSteps.title")}</CardTitle>
          <div className="flex flex-wrap gap-1">
            <Badge variant="info">{diagnosis.slowdown_shape.replace(/_/g, " ")}</Badge>
            <Badge
              variant={
                diagnosis.confidence === "high"
                  ? "active"
                  : diagnosis.confidence === "medium"
                    ? "caution"
                    : "info"
              }
            >
              {t("nextSteps.confidence", { level: diagnosis.confidence })}
            </Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="rounded-lg border border-border/70 bg-muted/15 p-3">
          <div className="mb-1 flex items-center gap-2">
            {diagnosis.slowdown_shape === "restricted_network" ? (
              <ShieldAlert className="size-4" />
            ) : diagnosis.slowdown_shape === "under_load_lag" ? (
              <Gauge className="size-4" />
            ) : (
              <ShieldCheck className="size-4" />
            )}
            <p className="text-sm font-medium">{t(plan.titleKey)}</p>
          </div>
          <p className="text-muted-foreground text-xs leading-relaxed">{t(plan.summaryKey)}</p>
          {recurrence && (
            <p className="text-muted-foreground mt-2 text-xs">
              {t("nextSteps.recurrence", {
                shape: recurrence.shape.replace(/_/g, " "),
                count: recurrence.count,
                window: Math.min(state.trends.length, 24),
              })}
            </p>
          )}
        </div>

        <div className="flex flex-wrap gap-2">
          {plan.actions.map((action, index) => (
            <Button
              key={`${action.page}-${action.labelKey}`}
              size="sm"
              variant={action.page === state.page ? "secondary" : "default"}
              onClick={() => state.navigate(action.page)}
            >
              {t(action.labelKey)}
              {plan.prioritized && index === 0 && (
                <Badge variant="recommended" className="ml-2">
                  {t("nextSteps.prioritized")}
                </Badge>
              )}
              <ArrowRight className="ml-1 size-3.5" />
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
