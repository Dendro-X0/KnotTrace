import { PathBreakPanel } from "@/components/PathBreakPanel";
import { IntegrityBoardPanel } from "@/components/IntegrityBoardPanel";
import { DiagnosticBundlePanel } from "@/components/DiagnosticBundlePanel";
import { LocalExplainPanel } from "@/components/LocalExplainPanel";
import { StatusHero } from "@/components/StatusHero";
import { Button } from "@/components/ui/button";
import type { CompanionState } from "@/hooks/useCompanion";

interface CompanionHomePageProps {
  state: CompanionState;
}

export function CompanionHomePage({ state }: CompanionHomePageProps) {
  const grade = state.report?.score.grade;
  const loading = state.bootstrapping && !state.report;
  const checking = state.checking;
  const lastReason = state.monitorStatus?.last_reason ?? "";
  const checkProfile = state.monitorStatus
    ? lastReason.startsWith("manual")
      ? "fast"
      : "full"
    : "fast";

  return (
    <div className="mx-auto flex w-full max-w-lg flex-col gap-3 px-1 pb-6">
      <header className="space-y-1 pt-1">
        <p className="text-2xl font-semibold tracking-tight">KnotTrace</p>
        <p className="text-muted-foreground text-sm leading-relaxed">
          Fast path check and redacted diagnostic export. Assists and Protect apply stay on desktop.
        </p>
      </header>

      <StatusHero
        loading={loading}
        grade={grade}
        score={state.report?.score.score}
        dnsIntegrity={state.report?.dns_integrity}
        checkProfile={checkProfile}
        summary={
          state.checkError ??
          state.report?.diagnosis?.summary ??
          state.report?.score.summary ??
          "Run a check to measure this path."
        }
      />

      <Button
        className="min-h-11 w-full"
        disabled={checking}
        onClick={() => void state.runCheck()}
      >
        {checking ? "Checking…" : state.report ? "Run check again" : "Run check"}
      </Button>

      <PathBreakPanel loading={loading || checking} report={state.report?.path_break} compact />
      <IntegrityBoardPanel loading={loading || checking} report={state.report?.integrity} compact />
      <DiagnosticBundlePanel state={state} />
      <LocalExplainPanel state={state} />
    </div>
  );
}
