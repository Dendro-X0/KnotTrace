import { useEffect, useState } from "react";
import { CheckCircle2, Copy, ExternalLink, FileSearch, FolderOpen, Loader2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type { CompanionState } from "@/hooks/useCompanion";
import {
  buildScriptRiskGuide,
  flattenScriptRiskHits,
  revealablePathForHit,
  type ScriptRiskHitKind,
} from "@/lib/scriptRiskGuide";
import { cn, invokeErrorMessage } from "@/lib/utils";
import type { SuspiciousActivityHit, SuspiciousActivitySnapshot } from "@/types";

interface ScriptRiskPanelProps {
  state: CompanionState;
}

function kindLabel(kind: ScriptRiskHitKind): string {
  if (kind === "persistence") return "Startup";
  if (kind === "mining") return "Process";
  return "Script file";
}

function isWindowsClient(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Win/i.test(navigator.userAgent) || /Win/i.test(navigator.platform);
}

async function copyText(value: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch {
    return false;
  }
}

async function revealPath(path: string): Promise<boolean> {
  try {
    const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
    await revealItemInDir(path);
    return true;
  } catch {
    return false;
  }
}

async function openStartupSettings(): Promise<boolean> {
  if (!isWindowsClient()) return false;
  try {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl("ms-settings:startupapps");
    return true;
  } catch {
    return false;
  }
}

function HitCard({
  kind,
  hit,
  onStatus,
}: {
  kind: ScriptRiskHitKind;
  hit: SuspiciousActivityHit;
  onStatus: (message: string) => void;
}) {
  const path = revealablePathForHit(hit);
  const copyTarget = path ?? hit.location ?? hit.name;

  return (
    <li className="rounded-xl border border-border/50 bg-background/40 px-3 py-2.5">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-1.5">
            <Badge variant="muted" className="normal-case">
              {kindLabel(kind)}
            </Badge>
            <p className="text-sm font-medium leading-snug">{hit.name}</p>
          </div>
          <p className="text-muted-foreground mt-0.5 truncate text-[0.68rem]" title={hit.location}>
            {hit.location}
          </p>
          {hit.detail && (
            <p className="text-muted-foreground mt-0.5 line-clamp-2 text-[0.68rem]">{hit.detail}</p>
          )}
        </div>
        <div className="flex shrink-0 flex-wrap gap-1.5">
          {path && (
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                void revealPath(path).then((ok) =>
                  onStatus(ok ? "Opened folder for this hit." : "Could not open that folder."),
                );
              }}
            >
              <FolderOpen className="size-3.5" aria-hidden />
              Reveal
            </Button>
          )}
          <Button
            size="sm"
            variant="secondary"
            onClick={() => {
              void copyText(copyTarget).then((ok) =>
                onStatus(ok ? "Copied to clipboard." : "Could not copy."),
              );
            }}
          >
            <Copy className="size-3.5" aria-hidden />
            Copy
          </Button>
        </div>
      </div>
    </li>
  );
}

export function ScriptRiskPanel({ state }: ScriptRiskPanelProps) {
  const [settings, setSettings] = useState<Awaited<
    ReturnType<CompanionState["getSuspiciousActivitySettings"]>
  > | null>(null);
  const [consent, setConsent] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [snapshot, setSnapshot] = useState<SuspiciousActivitySnapshot | null>(
    () => state.report?.integrity?.suspicious_activity ?? null,
  );

  useEffect(() => {
    let cancelled = false;
    void state
      .getSuspiciousActivitySettings()
      .then((next) => {
        if (!cancelled) {
          setSettings(next);
          setConsent(next.consent_ack);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(invokeErrorMessage(err, "Could not load script-risk settings."));
      });
    return () => {
      cancelled = true;
    };
  }, [state.getSuspiciousActivitySettings]);

  useEffect(() => {
    const fromReport = state.report?.integrity?.suspicious_activity;
    if (fromReport?.scanned) {
      setSnapshot(fromReport);
    }
  }, [state.report?.integrity?.suspicious_activity]);

  const enabled = settings?.enabled ?? false;
  const guide = buildScriptRiskGuide(snapshot);
  const hits = flattenScriptRiskHits(snapshot);
  const hitCount = hits.length;
  const canScan = Boolean(settings) && !scanning;
  const primaryLabel = !consent ? "Allow & scan" : snapshot?.scanned ? "Scan again" : "Scan this PC";

  const runScan = async () => {
    setScanning(true);
    setError(null);
    setStatus(null);
    try {
      const next = await state.setSuspiciousActivitySettings({
        enabled,
        consentAck: true,
      });
      setSettings(next);
      setConsent(true);
      const result = await state.runScriptRiskScan();
      setSnapshot(result);
      setStatus("Scan finished — review results and next steps below.");
    } catch (err: unknown) {
      setError(invokeErrorMessage(err, "Script risk scan failed."));
    } finally {
      setScanning(false);
    }
  };

  return (
    <section className="overflow-hidden rounded-xl border border-border/50 bg-card/70">
      <div className="space-y-3 px-4 py-4 sm:px-5">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="flex min-w-0 items-start gap-2.5">
            <FileSearch className="text-muted-foreground mt-0.5 size-3.5 shrink-0" aria-hidden />
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <h2 className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                  Script risk
                </h2>
                <Badge variant="info" className="normal-case">
                  Indicators only
                </Badge>
                {snapshot?.scanned && (
                  <Badge
                    variant={hitCount > 0 ? "caution" : "muted"}
                    className="normal-case"
                  >
                    {hitCount > 0
                      ? hitCount === 1
                        ? "1 indicator"
                        : `${hitCount} indicators`
                      : "No matches"}
                  </Badge>
                )}
              </div>
              <p className="text-muted-foreground mt-1 max-w-2xl text-sm leading-relaxed text-pretty">
                One click samples startup entries, processes, and a few user folders for mining-like
                or risky-script patterns. Not antivirus — KnotTrace will not delete or kill anything.
              </p>
            </div>
          </div>
        </div>

        <div className="flex flex-col gap-2 rounded-xl border border-border/45 bg-muted/10 px-3 py-3 sm:flex-row sm:items-center sm:justify-between">
          <p className="text-muted-foreground text-xs leading-relaxed sm:max-w-md">
            {consent
              ? "Permission is on for this PC. Run the bounded scan whenever you want a fresh sample."
              : "First scan also records your permission to read local startup, process, and script samples."}
          </p>
          <Button
            className="hub-cta shrink-0"
            disabled={!canScan}
            onClick={() => void runScan()}
          >
            {scanning ? (
              <>
                <Loader2 className="size-4 animate-spin" aria-hidden />
                Scanning…
              </>
            ) : (
              primaryLabel
            )}
          </Button>
        </div>

        {error && <p className="text-destructive text-xs">{error}</p>}
        {status && !error && (
          <p className="text-muted-foreground flex items-start gap-1.5 text-xs leading-relaxed" role="status">
            <CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-emerald-500" aria-hidden />
            {status}
          </p>
        )}

        <div className="flex items-center justify-between gap-3 border-t border-border/35 pt-3">
          <Label htmlFor="script-risk-enabled" className="text-muted-foreground text-xs">
            Also sample during health checks
          </Label>
          <Switch
            id="script-risk-enabled"
            checked={enabled}
            disabled={!settings}
            onCheckedChange={(value) => {
              if (value && !consent) {
                setError("Run Allow & scan once first, or turn this on after a scan.");
                return;
              }
              void state
                .setSuspiciousActivitySettings({
                  enabled: value,
                  consentAck: value ? consent : (settings?.consent_ack ?? false),
                })
                .then((next) => {
                  setSettings(next);
                  setConsent(next.consent_ack);
                  setError(null);
                })
                .catch((err: unknown) =>
                  setError(invokeErrorMessage(err, "Could not save script-risk settings.")),
                );
            }}
          />
        </div>
      </div>

      {!snapshot?.scanned ? (
        <div className="border-t border-border/40 px-4 py-4 sm:px-5">
          <p className="text-muted-foreground text-xs leading-relaxed">
            No scan yet. Press <span className="text-foreground font-medium">{primaryLabel}</span>{" "}
            to start — results and next steps appear here.
          </p>
        </div>
      ) : (
        <div className="space-y-4 border-t border-border/40 px-4 py-4 sm:px-5">
          <div>
            <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
              Results
            </p>
            <p className="mt-1 text-sm leading-relaxed text-pretty">{snapshot.summary}</p>
          </div>

          {hitCount > 0 ? (
            <ul className="grid gap-2">
              {hits.map(({ kind, hit }) => (
                <HitCard
                  key={`${kind}-${hit.name}-${hit.location}`}
                  kind={kind}
                  hit={hit}
                  onStatus={setStatus}
                />
              ))}
            </ul>
          ) : (
            <p className="text-muted-foreground text-xs leading-relaxed">
              No matches in this sample. That does not mean the PC is malware-free.
            </p>
          )}

          {guide && (
            <div className="rounded-xl border border-border/45 bg-muted/10 px-3 py-3">
              <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                What to do next
              </p>
              <p className="mt-1 text-sm font-medium leading-snug">{guide.headline}</p>
              <ol className="mt-2.5 grid gap-2.5">
                {guide.steps.map((step, index) => (
                  <li key={step.id} className="flex gap-2.5">
                    <span
                      className={cn(
                        "bg-background/60 text-muted-foreground flex size-5 shrink-0 items-center justify-center rounded-full border border-border/50 text-[0.65rem] font-semibold",
                      )}
                    >
                      {index + 1}
                    </span>
                    <div className="min-w-0 flex-1 space-y-1.5">
                      <p className="text-sm font-medium leading-snug">{step.title}</p>
                      <p className="text-muted-foreground text-xs leading-relaxed text-pretty">
                        {step.detail}
                      </p>
                      <div className="flex flex-wrap gap-1.5">
                        {step.copyText && (
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => {
                              void copyText(step.copyText!).then((ok) =>
                                setStatus(ok ? "Copied guidance detail." : "Could not copy."),
                              );
                            }}
                          >
                            <Copy className="size-3.5" aria-hidden />
                            Copy detail
                          </Button>
                        )}
                        {step.openStartupSettings && isWindowsClient() && (
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => {
                              void openStartupSettings().then((ok) =>
                                setStatus(
                                  ok
                                    ? "Opened Windows Startup apps."
                                    : "Could not open Startup settings.",
                                ),
                              );
                            }}
                          >
                            <ExternalLink className="size-3.5" aria-hidden />
                            Open Startup apps
                          </Button>
                        )}
                      </div>
                    </div>
                  </li>
                ))}
              </ol>
              <p className="text-muted-foreground mt-3 border-t border-border/35 pt-2.5 text-[0.68rem] leading-relaxed text-pretty">
                {guide.honesty}
              </p>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
