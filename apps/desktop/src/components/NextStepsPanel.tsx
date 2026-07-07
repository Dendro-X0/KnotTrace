import { ArrowRight, Gauge, ShieldAlert, ShieldCheck, Workflow } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { CompanionState } from "@/hooks/useCompanion";
import type { PageId } from "@/types";

interface NextStepsPanelProps {
  state: CompanionState;
}

interface NextStepAction {
  label: string;
  page: PageId;
}

interface NextStepPlan {
  title: string;
  summary: string;
  actions: NextStepAction[];
}

interface RecurrenceInsight {
  shape: string;
  count: number;
}

function buildPlan(state: CompanionState): NextStepPlan | null {
  const diagnosis = state.report?.diagnosis;
  if (!diagnosis) return null;

  switch (diagnosis.slowdown_shape) {
    case "page_start":
      return {
        title: "Start with DNS and sign-in checks",
        summary:
          "This looks more like a page-start delay than a raw bandwidth issue. Check DNS integrity first, then confirm the path is not behind a hotspot sign-in or proxy delay.",
        actions: [
          { label: "Open DNS Assist", page: "dns" },
          { label: "Review Network details", page: "network" },
          { label: "Check Protect status", page: "protect" },
        ],
      };
    case "under_load_lag":
      return {
        title: "Focus on latency under load",
        summary:
          "The path likely has enough bandwidth, but it becomes sluggish during downloads or uploads. Review the stability probes and compare with a manual throughput test.",
        actions: [
          { label: "Open Network page", page: "network" },
          { label: "Save a benchmark", page: "overview" },
        ],
      };
    case "partial_site_failure":
      return {
        title: "Compare the current path",
        summary:
          "Some sites appear to fail only on this route. Check proxy or tunnel behavior, then compare path-specific guidance on the Network page.",
        actions: [
          { label: "Open Connect Assist", page: "connect" },
          { label: "Open Network page", page: "network" },
        ],
      };
    case "restricted_network":
      return {
        title: "Verify network access before tuning",
        summary:
          "This looks like a guest, public, or captive network. Finish hotspot sign-in first, then re-check before making changes.",
        actions: [
          { label: "Open Protect page", page: "protect" },
          { label: "Open Network page", page: "network" },
        ],
      };
    case "tunnel_overhead":
      return {
        title: "Review tunnel overhead",
        summary:
          "A VPN, proxy, or Tor path may be adding latency or causing path-specific failures. Compare recommendations before switching anything manually.",
        actions: [
          { label: "Open Connect Assist", page: "connect" },
          { label: "Open Network page", page: "network" },
        ],
      };
    case "link_local_issue":
      return {
        title: "Check the local link first",
        summary:
          "The bottleneck looks close to your device or router. Review gateway latency, Wi-Fi vs Ethernet, and physical link quality before blaming the wider internet.",
        actions: [
          { label: "Open Network page", page: "network" },
          { label: "Open Protect page", page: "protect" },
        ],
      };
    case "general_degradation":
    default:
      return {
        title: "Review the latest diagnosis",
        summary:
          "The current results do not point to a single strong slowdown shape. Review diagnosis hints first, then inspect Network details for path clues.",
        actions: [
          { label: "Open Network page", page: "network" },
          { label: "Open Protect page", page: "protect" },
        ],
      };
  }
}

function prioritizeActionsForRecurrence(
  actions: NextStepAction[],
  recurrence: RecurrenceInsight | null,
): { actions: NextStepAction[]; prioritized: boolean } {
  if (!recurrence || recurrence.count < 3) return { actions, prioritized: false };

  const preferredFirstPageByShape: Record<string, PageId> = {
    page_start: "dns",
    partial_site_failure: "connect",
    restricted_network: "protect",
    under_load_lag: "network",
    tunnel_overhead: "connect",
    link_local_issue: "network",
  };

  const preferred = preferredFirstPageByShape[recurrence.shape];
  if (!preferred) return { actions, prioritized: false };

  const copy = [...actions];
  const idx = copy.findIndex((action) => action.page === preferred);
  if (idx <= 0) return { actions: copy, prioritized: false };

  const [target] = copy.splice(idx, 1);
  copy.unshift(target);
  return { actions: copy, prioritized: true };
}

export function NextStepsPanel({ state }: NextStepsPanelProps) {
  const loading = state.bootstrapping && !state.report;
  const plan = buildPlan(state);
  const diagnosis = state.report?.diagnosis;
  const recurrence = (() => {
    const latestShape = diagnosis?.slowdown_shape;
    if (!latestShape || latestShape === "general_degradation") return null;
    const recent = state.trends.slice(-24);
    const count = recent.filter((point) => point.slowdown_shape === latestShape).length;
    if (count < 2) return null;
    return { shape: latestShape, count } satisfies RecurrenceInsight;
  })();
  const prioritizedPlan = plan
    ? prioritizeActionsForRecurrence(plan.actions, recurrence)
    : { actions: [], prioritized: false };
  const actions = prioritizedPlan.actions;

  if (loading) {
    return (
      <Card className="border-border/70 bg-card/80">
        <CardHeader className="pb-2">
          <CardTitle>What to do next</CardTitle>
        </CardHeader>
        <CardContent className="text-muted-foreground text-sm">Analyzing your latest check…</CardContent>
      </Card>
    );
  }

  if (!plan || !diagnosis) {
    return (
      <EmptyState
        icon={Workflow}
        title="Next steps pending"
        description="Run a health check to get a symptom-specific next-step plan."
      />
    );
  }

  return (
    <Card className="border-border/70 bg-card/80">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle>What to do next</CardTitle>
          <div className="flex flex-wrap gap-1">
            <Badge variant="info">{diagnosis.slowdown_shape.replace(/_/g, " ")}</Badge>
            <Badge variant={diagnosis.confidence === "high" ? "active" : diagnosis.confidence === "medium" ? "caution" : "info"}>
              {diagnosis.confidence} confidence
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
            <p className="text-sm font-medium">{plan.title}</p>
          </div>
          <p className="text-muted-foreground text-xs leading-relaxed">{plan.summary}</p>
          {recurrence && (
            <p className="text-muted-foreground mt-2 text-xs">
              Recurring pattern: <span className="font-medium">{recurrence.shape.replace(/_/g, " ")}</span>{" "}
              appeared in <span className="font-medium">{recurrence.count}</span> of your last{" "}
              <span className="font-medium">{Math.min(state.trends.length, 24)}</span> checks.
            </p>
          )}
        </div>

        <div className="flex flex-wrap gap-2">
          {actions.map((action, index) => (
            <Button
              key={`${action.page}-${action.label}`}
              size="sm"
              variant={action.page === state.page ? "secondary" : "default"}
              onClick={() => state.navigate(action.page)}
            >
              {action.label}
              {prioritizedPlan.prioritized && index === 0 && (
                <Badge variant="recommended" className="ml-2">
                  Prioritized
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
