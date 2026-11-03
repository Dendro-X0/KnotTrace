import { useEffect, useState, type ReactNode } from "react";
import { AlertTriangle, History, Shield, ShieldAlert, ShieldCheck } from "lucide-react";

import { ScriptRiskPanel } from "@/components/ScriptRiskPanel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { statusCardClass } from "@/components/shared";
import { useUiDensity } from "@/hooks/useUiDensity";
import { cn } from "@/lib/utils";
import type { CompanionState } from "@/hooks/useCompanion";
import type {
  AutorunSettings,
  ProcessEgressSettings,
  ProtectAction,
  ProtectSettings,
} from "@/types";

interface ProtectPageProps {
  state: CompanionState;
}

type BooleanProtectKey = {
  [K in keyof ProtectSettings]: ProtectSettings[K] extends boolean ? K : never;
}[keyof ProtectSettings];

interface ToggleDef {
  key: BooleanProtectKey;
  label: string;
  hint?: string;
}

const CORE_TOGGLES: ToggleDef[] = [
  { key: "enabled", label: "Smart protect", hint: "Monitor trust and connection quality" },
  { key: "auto_apply_dns", label: "Auto-improve DNS", hint: "Apply faster DNS on untrusted or poor networks" },
  {
    key: "auto_apply_connect",
    label: "Auto-switch proxy node",
    hint: "Opt-in: pick a faster node when Mihomo/sing-box is available",
  },
  {
    key: "auto_apply_on_untrusted_only",
    label: "Auto-fix only when needed",
    hint: "Limit automatic changes to untrusted or poor connections",
  },
];

const RECOVERY_TOGGLES: ToggleDef[] = [
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
];

const NOTIFY_TOGGLES: ToggleDef[] = [
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

function trustHeadline(trust: "trusted" | "caution" | "untrusted" | undefined) {
  if (trust === "untrusted") return "Unfamiliar network";
  if (trust === "caution") return "Use caution";
  if (trust === "trusted") return "Looks familiar";
  return "Checking…";
}

function firstSentence(text: string): string {
  const trimmed = text.trim();
  const match = trimmed.match(/^(.+?[.!?])(?:\s|$)/);
  return match?.[1] ?? trimmed;
}

function trustGrade(trust: "trusted" | "caution" | "untrusted" | undefined) {
  if (trust === "untrusted") return "poor" as const;
  if (trust === "caution") return "fair" as const;
  if (trust === "trusted") return "good" as const;
  return undefined;
}

function ToggleRow({
  toggle,
  checked,
  onChange,
}: {
  toggle: ToggleDef;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  const id = `protect-${toggle.key}`;
  return (
    <div className="flex items-center justify-between gap-3 rounded-xl border border-border/50 bg-muted/15 px-3 py-2.5">
      <div className="min-w-0">
        <Label htmlFor={id} className="text-xs leading-snug">
          {toggle.label}
        </Label>
        {toggle.hint && <p className="text-muted-foreground mt-0.5 text-[0.68rem]">{toggle.hint}</p>}
      </div>
      <Switch id={id} checked={checked} onCheckedChange={onChange} />
    </div>
  );
}

function SettingsSection({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section className="grid gap-2">
      <div>
        <h2 className="text-sm font-medium">{title}</h2>
        {description && (
          <p className="text-muted-foreground mt-0.5 text-[0.68rem] leading-relaxed">{description}</p>
        )}
      </div>
      <div className="grid grid-cols-1 gap-2 xl:grid-cols-2">{children}</div>
    </section>
  );
}

export function ProtectPage({ state }: ProtectPageProps) {
  const { isQuick } = useUiDensity();
  const status = state.protectStatus;
  const loading = state.bootstrapping && !status && !state.protectError;
  const alerts = status?.alerts ?? [];
  const log = [...(state.autoProtectLog ?? [])].reverse();
  const previewLog = isQuick ? log.slice(0, 2) : log.slice(0, 6);
  const hiddenLogCount = Math.max(0, log.length - previewLog.length);

  const [processEgress, setProcessEgress] = useState<ProcessEgressSettings | null>(null);
  const [processEgressError, setProcessEgressError] = useState<string | null>(null);
  const [consentChecked, setConsentChecked] = useState(false);
  const [autoruns, setAutoruns] = useState<AutorunSettings | null>(null);
  const [autorunsError, setAutorunsError] = useState<string | null>(null);
  const [autorunsConsent, setAutorunsConsent] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void state
      .getProcessEgressSettings()
      .then((settings) => {
        if (!cancelled) {
          setProcessEgress(settings);
          setConsentChecked(settings.consent_ack);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setProcessEgressError(error instanceof Error ? error.message : String(error));
        }
      });
    void state
      .getAutorunSettings()
      .then((settings) => {
        if (!cancelled) {
          setAutoruns(settings);
          setAutorunsConsent(settings.consent_ack);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setAutorunsError(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [state.getProcessEgressSettings, state.getAutorunSettings]);

  const saveToggle = (key: BooleanProtectKey, value: boolean) => {
    void state.saveProtectSettings({ [key]: value });
  };

  const grade = trustGrade(status?.trust_level);
  const TrustIcon =
    status?.trust_level === "untrusted"
      ? ShieldAlert
      : status?.trust_level === "caution"
        ? AlertTriangle
        : ShieldCheck;

  return (
    <div className="flex flex-col gap-3">
      <Card className={cn(statusCardClass(grade), "overflow-hidden")}>
        <CardContent className="flex flex-col gap-3 px-5 py-5 sm:px-6">
          {loading ? (
            <div className="flex items-center gap-4">
              <Skeleton className="size-14 rounded-2xl" />
              <div className="flex-1 space-y-2">
                <Skeleton className="h-7 w-48" />
                <Skeleton className="h-4 w-full max-w-lg" />
              </div>
            </div>
          ) : (
            <>
              <div className="flex min-w-0 items-start gap-3 sm:gap-4">
                <div className="bg-background/40 flex size-12 shrink-0 items-center justify-center rounded-2xl border border-border/50 sm:size-14">
                  <TrustIcon className="size-6 sm:size-7" />
                </div>
                <div className="min-w-0 flex-1 space-y-1.5">
                  <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                    <p className="text-2xl font-semibold tracking-tight sm:text-3xl">
                      {trustHeadline(status?.trust_level)}
                    </p>
                    {alerts.length > 0 ? (
                      <Badge variant="caution">{alerts.length} to review</Badge>
                    ) : (
                      <Badge variant="muted">All clear here</Badge>
                    )}
                  </div>
                  <p className="text-muted-foreground max-w-2xl text-sm leading-relaxed">
                    {state.protectError ??
                      status?.summary ??
                      "Run a health check to refresh how this network looks."}
                  </p>
                </div>
              </div>

              {alerts.length > 0 && (
                <ul className="grid gap-1.5 border-t border-border/40 pt-3">
                  {alerts.map((alert) => {
                    const usefulActions = alert.actions.filter(
                      (action) =>
                        action.kind !== "run_check" &&
                        action.label.toLowerCase() !== "run health check",
                    );
                    return (
                      <li key={`${alert.title}-${alert.message}`}>
                        <details className="group rounded-xl border border-border/40 bg-background/30 open:bg-background/40">
                          <summary className="flex cursor-pointer list-none items-center gap-2 px-3 py-2.5 [&::-webkit-details-marker]:hidden">
                            <span
                              className={cn(
                                "size-1.5 shrink-0 rounded-full",
                                alert.level === "critical"
                                  ? "bg-rose-500"
                                  : alert.level === "warning"
                                    ? "bg-amber-500"
                                    : "bg-sky-500",
                              )}
                              aria-hidden
                            />
                            <span className="min-w-0 flex-1 text-sm font-medium leading-snug">
                              {alert.title}
                            </span>
                            <span className="text-muted-foreground text-[0.65rem] group-open:hidden">
                              Details
                            </span>
                          </summary>
                          <div className="space-y-2 border-t border-border/30 px-3 py-2.5">
                            <p className="text-muted-foreground text-xs leading-relaxed">
                              {isQuick ? firstSentence(alert.message) : alert.message}
                            </p>
                            {usefulActions.length > 0 && (
                              <div className="flex flex-wrap gap-2">
                                {usefulActions.map((action) => (
                                  <Button
                                    key={action.label}
                                    size="sm"
                                    variant="secondary"
                                    onClick={() =>
                                      state.handleProtectAction(
                                        action.kind as ProtectAction["kind"],
                                      )
                                    }
                                  >
                                    {action.label}
                                  </Button>
                                ))}
                              </div>
                            )}
                          </div>
                        </details>
                      </li>
                    );
                  })}
                </ul>
              )}
            </>
          )}
        </CardContent>
      </Card>

      <ScriptRiskPanel state={state} />

      <details className="rounded-xl border border-border/50 bg-card/50 open:bg-card/70">
        <summary className="flex cursor-pointer list-none items-center justify-between gap-2 px-3.5 py-3 text-sm [&::-webkit-details-marker]:hidden">
          <span className="flex items-center gap-2">
            <History className="text-muted-foreground size-3.5" />
            <span className="font-medium">Recent activity</span>
          </span>
          <Badge variant="muted">{log.length}</Badge>
        </summary>
        <div className="space-y-2 border-t border-border/40 px-3.5 py-3">
          {state.autoProtectLogError ? (
            <p className="text-muted-foreground text-xs">{state.autoProtectLogError}</p>
          ) : log.length === 0 ? (
            <p className="text-muted-foreground text-xs leading-relaxed">
              No automatic DNS or proxy changes yet.
            </p>
          ) : (
            <>
              <ul className="grid gap-1.5">
                {previewLog.map((entry) => (
                  <li
                    key={`${entry.timestamp}-${entry.kind}-${entry.message}`}
                    className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 rounded-lg bg-muted/10 px-2.5 py-2"
                  >
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-1.5">
                        <span className="text-xs font-medium capitalize">{entry.kind}</span>
                        <Badge variant={entry.success ? "active" : "muted"} className="text-[0.6rem]">
                          {entry.success ? "applied" : "skipped"}
                        </Badge>
                      </div>
                      <p className="text-muted-foreground mt-0.5 line-clamp-1 text-[0.68rem]">
                        {entry.message}
                      </p>
                    </div>
                    <span className="text-muted-foreground shrink-0 text-[0.65rem]">
                      {new Date(entry.timestamp).toLocaleString()}
                    </span>
                  </li>
                ))}
              </ul>
              {hiddenLogCount > 0 && (
                <p className="text-muted-foreground text-[0.68rem]">
                  +{hiddenLogCount} older {hiddenLogCount === 1 ? "entry" : "entries"} — switch to
                  Advanced for the full log.
                </p>
              )}
            </>
          )}
        </div>
      </details>

      <details
        className="rounded-xl border border-border/60 bg-card/70"
        open={!isQuick}
      >
        <summary
          className={cn(
            "flex cursor-pointer list-none items-center gap-2 px-3.5 py-3 text-sm font-medium [&::-webkit-details-marker]:hidden",
            !isQuick && "pointer-events-none",
          )}
        >
          Protection preferences
          {isQuick && (
            <span className="text-muted-foreground text-xs font-normal">Smart protect & autos</span>
          )}
        </summary>
        <div className="flex flex-col gap-3 border-t border-border/40 px-3.5 py-3.5">
          {loading ? (
            <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
              {Array.from({ length: 4 }).map((_, index) => (
                <Skeleton key={index} className="h-12 rounded-xl" />
              ))}
            </div>
          ) : (
            <>
              <SettingsSection
                title="Automatic actions"
                description="What KnotTrace may change when Smart protect is on."
              >
                {CORE_TOGGLES.map((toggle) => (
                  <ToggleRow
                    key={toggle.key}
                    toggle={toggle}
                    checked={status?.settings[toggle.key] ?? false}
                    onChange={(value) => saveToggle(toggle.key, value)}
                  />
                ))}
                {!isQuick &&
                  RECOVERY_TOGGLES.map((toggle) => (
                    <ToggleRow
                      key={toggle.key}
                      toggle={toggle}
                      checked={status?.settings[toggle.key] ?? false}
                      onChange={(value) => saveToggle(toggle.key, value)}
                    />
                  ))}
              </SettingsSection>

              {!isQuick && (
                <>
                  <SettingsSection
                    title="Notifications"
                    description="When the OS should interrupt you — independent of monitoring."
                  >
                    {NOTIFY_TOGGLES.map((toggle) => (
                      <ToggleRow
                        key={toggle.key}
                        toggle={toggle}
                        checked={status?.settings[toggle.key] ?? false}
                        onChange={(value) => saveToggle(toggle.key, value)}
                      />
                    ))}
                  </SettingsSection>

                  {(status?.settings.quiet_hours_enabled ?? false) && (
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                      <div className="grid gap-1.5 rounded-xl border border-border/50 bg-muted/15 px-3 py-2.5">
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
                      <div className="grid gap-1.5 rounded-xl border border-border/50 bg-muted/15 px-3 py-2.5">
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
                      </div>
                    </div>
                  )}

                  <section className="grid gap-2 rounded-xl border border-border/50 bg-muted/10 px-3 py-3">
                    <div className="flex items-center gap-2">
                      <Shield className="text-muted-foreground size-3.5" />
                      <h2 className="text-sm font-medium">Process ↔ egress (opt-in)</h2>
                    </div>
                    <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
                      Sample OS connections matching current egress/proxy; list process names as
                      indicators only. Off by default.
                    </p>
                    {processEgressError && (
                      <p className="text-muted-foreground text-xs">{processEgressError}</p>
                    )}
                    <label className="flex items-start gap-2 text-xs">
                      <input
                        type="checkbox"
                        className="mt-0.5"
                        checked={consentChecked}
                        onChange={(event) => setConsentChecked(event.target.checked)}
                      />
                      <span>
                        I understand KnotTrace will read local connection/process names on this device
                        only.
                      </span>
                    </label>
                    <div className="flex items-center justify-between gap-3">
                      <Label htmlFor="process-egress-enabled" className="text-xs">
                        Enable process ↔ egress sampling
                      </Label>
                      <Switch
                        id="process-egress-enabled"
                        checked={processEgress?.enabled ?? false}
                        disabled={!processEgress}
                        onCheckedChange={(value) => {
                          if (value && !consentChecked) {
                            setProcessEgressError("Ack the consent note before enabling.");
                            return;
                          }
                          void state
                            .setProcessEgressSettings({
                              enabled: value,
                              consentAck: value
                                ? consentChecked
                                : (processEgress?.consent_ack ?? false),
                            })
                            .then((next) => {
                              setProcessEgress(next);
                              setConsentChecked(next.consent_ack);
                              setProcessEgressError(null);
                            })
                            .catch((error: unknown) => {
                              setProcessEgressError(
                                error instanceof Error ? error.message : String(error),
                              );
                            });
                        }}
                      />
                    </div>
                  </section>

                  <section className="grid gap-2 rounded-xl border border-border/50 bg-muted/10 px-3 py-3">
                    <h2 className="text-sm font-medium">Startup network agents (opt-in)</h2>
                    <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
                      Flag startup names that look like VPN/proxy clients as indicators. Does not
                      disable anything.
                    </p>
                    {autorunsError && <p className="text-muted-foreground text-xs">{autorunsError}</p>}
                    <label className="flex items-start gap-2 text-xs">
                      <input
                        type="checkbox"
                        className="mt-0.5"
                        checked={autorunsConsent}
                        onChange={(event) => setAutorunsConsent(event.target.checked)}
                      />
                      <span>
                        I understand KnotTrace will read local startup/autorun entry names on this
                        device only.
                      </span>
                    </label>
                    <div className="flex items-center justify-between gap-3">
                      <Label htmlFor="autoruns-enabled" className="text-xs">
                        Enable startup network-agent scan
                      </Label>
                      <Switch
                        id="autoruns-enabled"
                        checked={autoruns?.enabled ?? false}
                        disabled={!autoruns}
                        onCheckedChange={(value) => {
                          if (value && !autorunsConsent) {
                            setAutorunsError("Ack the consent note before enabling.");
                            return;
                          }
                          void state
                            .setAutorunSettings({
                              enabled: value,
                              consentAck: value ? autorunsConsent : (autoruns?.consent_ack ?? false),
                            })
                            .then((next) => {
                              setAutoruns(next);
                              setAutorunsConsent(next.consent_ack);
                              setAutorunsError(null);
                            })
                            .catch((error: unknown) => {
                              setAutorunsError(
                                error instanceof Error ? error.message : String(error),
                              );
                            });
                        }}
                      />
                    </div>
                  </section>
                </>
              )}

              {isQuick && (
                <p className="text-muted-foreground text-xs leading-relaxed">
                  Recovery rules, notifications, and deeper integrity sampling are in{" "}
                  <span className="text-foreground font-medium">Advanced</span>.
                </p>
              )}
            </>
          )}

          {state.autoProtectNote && (
            <p className="text-muted-foreground rounded-lg border border-border/50 bg-muted/10 px-3 py-2 text-xs">
              {state.autoProtectNote}
            </p>
          )}
        </div>
      </details>
    </div>
  );
}
