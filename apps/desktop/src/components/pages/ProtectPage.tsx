import { Shield, ShieldCheck, History } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { FeaturePage } from "@/components/FeaturePage";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import type { CompanionState } from "@/hooks/useCompanion";
import type { ProtectAction, ProtectSettings } from "@/types";

interface ProtectPageProps {
  state: CompanionState;
}

function protectBadgeVariant(trust: "trusted" | "caution" | "untrusted" | undefined) {
  if (trust === "untrusted") return "untrusted" as const;
  if (trust === "caution") return "caution" as const;
  return "active" as const;
}

type BooleanProtectKey = {
  [K in keyof ProtectSettings]: ProtectSettings[K] extends boolean ? K : never;
}[keyof ProtectSettings];

const TOGGLES: Array<{
  key: BooleanProtectKey;
  label: string;
  hint?: string;
}> = [
  { key: "enabled", label: "Smart protect", hint: "Monitor trust and connection quality" },
  { key: "auto_apply_dns", label: "Auto-improve DNS", hint: "Apply faster DNS on untrusted or poor networks" },
  { key: "auto_apply_connect", label: "Auto-switch proxy node", hint: "Opt-in: pick a faster node when Mihomo/sing-box is available" },
  {
    key: "auto_recover_dns_integrity",
    label: "Recover from DNS poisoning",
    hint: "Apply trusted DNS when integrity checks detect hijacking",
  },
  {
    key: "auto_recover_site_access",
    label: "Recover site access",
    hint: "Opt-in: switch proxy nodes when verification sites fail over HTTPS",
  },
  {
    key: "auto_apply_on_untrusted_only",
    label: "Auto-fix only when needed",
    hint: "Limit automatic changes to untrusted or poor connections",
  },
  {
    key: "do_not_disturb",
    label: "Do Not Disturb",
    hint: "Complete silence — no system notifications; monitoring and auto-protect still run",
  },
  {
    key: "notify_digest_only",
    label: "Notification digest",
    hint: "Combine alerts into one summary every few minutes instead of one toast each",
  },
  {
    key: "quiet_hours_enabled",
    label: "Quiet hours",
    hint: "Silence OS notifications during a local time window (supports overnight)",
  },
  { key: "notify_on_grade_drop", label: "Notify on score drop" },
  { key: "notify_on_untrusted_network", label: "Notify on unfamiliar networks" },
  { key: "notify_on_degraded", label: "Notify on poor connection" },
];

export function ProtectPage({ state }: ProtectPageProps) {
  const status = state.protectStatus;
  const loading = state.bootstrapping && !status && !state.protectError;

  return (
    <FeaturePage
      title="Protect"
      description={
        state.protectError ??
        status?.summary ??
        "Monitoring trust level and connection degradation."
      }
      icon={Shield}
      badge={
        loading
          ? undefined
          : {
              label: status?.trust_level ?? "checking",
              variant: protectBadgeVariant(status?.trust_level),
            }
      }
      loading={loading}
    >
      {loading ? (
        <Skeleton className="h-24 rounded-lg" />
      ) : (status?.alerts ?? []).length === 0 ? (
        <EmptyState
          icon={ShieldCheck}
          title="No active protect alerts"
          description="Your current network trust level looks stable. Alerts will appear here when action is recommended."
        />
      ) : (
        <ScrollArea className="max-h-[min(22rem,50vh)]">
          <ul className="grid gap-2">
            {status?.alerts.map((alert) => (
              <li
                key={`${alert.title}-${alert.message}`}
                className="rounded-lg border border-border/70 bg-muted/20 p-3"
              >
                <div className="mb-1 flex items-center gap-2">
                  <h3 className="text-sm font-medium">{alert.title}</h3>
                  <Badge variant={alert.level === "critical" ? "poor" : alert.level === "warning" ? "caution" : "info"}>
                    {alert.level}
                  </Badge>
                </div>
                <p className="text-muted-foreground text-xs">{alert.message}</p>
                <div className="mt-2 flex flex-wrap gap-2">
                  {alert.actions.map((action) => (
                    <Button
                      key={action.label}
                      size="sm"
                      variant="secondary"
                      onClick={() => state.handleProtectAction(action.kind as ProtectAction["kind"])}
                    >
                      {action.label}
                    </Button>
                  ))}
                </div>
              </li>
            ))}
          </ul>
        </ScrollArea>
      )}

      <Separator className="shrink-0" />

      <section className="grid gap-2">
        <div className="flex items-center gap-2">
          <History className="text-muted-foreground size-4" />
          <h2 className="text-sm font-medium">Recent automatic actions</h2>
        </div>
        {state.autoProtectLogError ? (
          <p className="text-muted-foreground text-xs">{state.autoProtectLogError}</p>
        ) : (state.autoProtectLog ?? []).length === 0 ? (
          <p className="text-muted-foreground rounded-lg border border-border/60 bg-muted/15 px-3 py-2 text-xs">
            No automatic DNS or proxy changes yet. When auto-protect runs, actions appear here with a rollback hint.
          </p>
        ) : (
          <ScrollArea className="max-h-[min(16rem,40vh)]">
            <ul className="grid gap-2">
              {[...state.autoProtectLog].reverse().map((entry) => (
                <li
                  key={`${entry.timestamp}-${entry.kind}-${entry.message}`}
                  className="rounded-lg border border-border/70 bg-muted/20 p-3"
                >
                  <div className="mb-1 flex flex-wrap items-center gap-2">
                    <span className="text-sm font-medium capitalize">{entry.kind}</span>
                    <Badge variant={entry.success ? "active" : "poor"}>
                      {entry.success ? "applied" : "skipped"}
                    </Badge>
                    <span className="text-muted-foreground text-[0.68rem]">
                      {new Date(entry.timestamp).toLocaleString()}
                    </span>
                  </div>
                  <p className="text-muted-foreground text-xs">{entry.message}</p>
                  <p className="text-muted-foreground mt-1 text-[0.68rem]">
                    Trigger: {entry.trigger.replace(/_/g, " ")} · {entry.rollback_hint}
                  </p>
                </li>
              ))}
            </ul>
          </ScrollArea>
        )}
      </section>

      <Separator className="shrink-0" />

      {loading ? (
        <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
          {Array.from({ length: 4 }).map((_, index) => (
            <Skeleton key={index} className="h-12 rounded-xl" />
          ))}
        </div>
      ) : (
        <>
          <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
            {TOGGLES.map((toggle) => {
              const id = `protect-${toggle.key}`;
              const checked = status?.settings[toggle.key] ?? false;
              return (
                <div
                  key={toggle.key}
                  className="flex items-center justify-between gap-3 rounded-xl border border-border/60 bg-muted/20 px-3 py-2.5"
                >
                  <div className="min-w-0">
                    <Label htmlFor={id} className="text-xs leading-snug">
                      {toggle.label}
                    </Label>
                    {toggle.hint && (
                      <p className="text-muted-foreground mt-0.5 text-[0.68rem]">{toggle.hint}</p>
                    )}
                  </div>
                  <Switch
                    id={id}
                    checked={checked}
                    onCheckedChange={(value) => {
                      void state.saveProtectSettings({ [toggle.key]: value });
                    }}
                  />
                </div>
              );
            })}
          </div>

          {(status?.settings.quiet_hours_enabled ?? false) && (
            <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div className="grid gap-1.5 rounded-xl border border-border/60 bg-muted/20 px-3 py-2.5">
                <Label htmlFor="protect-quiet-start" className="text-xs">
                  Quiet hours start
                </Label>
                <Input
                  id="protect-quiet-start"
                  type="time"
                  value={status?.settings.quiet_hours_start ?? "22:00"}
                  onChange={(event) => {
                    void state.saveProtectSettings({
                      quiet_hours_start: event.target.value || "22:00",
                    });
                  }}
                />
              </div>
              <div className="grid gap-1.5 rounded-xl border border-border/60 bg-muted/20 px-3 py-2.5">
                <Label htmlFor="protect-quiet-end" className="text-xs">
                  Quiet hours end
                </Label>
                <Input
                  id="protect-quiet-end"
                  type="time"
                  value={status?.settings.quiet_hours_end ?? "07:00"}
                  onChange={(event) => {
                    void state.saveProtectSettings({
                      quiet_hours_end: event.target.value || "07:00",
                    });
                  }}
                />
                <p className="text-muted-foreground text-[0.68rem]">
                  Overnight windows work (e.g. 22:00 → 07:00).
                </p>
              </div>
            </div>
          )}
        </>
      )}

      {state.autoProtectNote && (
        <p className="text-muted-foreground rounded-lg border border-border/60 bg-muted/15 px-3 py-2 text-xs">
          {state.autoProtectNote}
        </p>
      )}
    </FeaturePage>
  );
}
