import { useEffect, useState } from "react";
import { FileSearch, Loader2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type { CompanionState } from "@/hooks/useCompanion";
import type { SuspiciousActivityHit, SuspiciousActivitySnapshot } from "@/types";
import { invokeErrorMessage } from "@/lib/utils";

interface ScriptRiskPanelProps {
  state: CompanionState;
}

function HitList({ title, hits }: { title: string; hits: SuspiciousActivityHit[] }) {
  if (hits.length === 0) return null;
  return (
    <div className="space-y-1.5">
      <p className="text-muted-foreground text-[0.65rem] font-medium tracking-wide uppercase">
        {title}
      </p>
      <ul className="grid gap-1.5">
        {hits.map((hit) => (
          <li
            key={`${hit.name}-${hit.location}`}
            className="rounded-lg border border-border/50 bg-background/40 px-2.5 py-2"
          >
            <p className="text-sm font-medium leading-snug">{hit.name}</p>
            <p className="text-muted-foreground truncate text-[0.68rem]">{hit.location}</p>
            {hit.detail && (
              <p className="text-muted-foreground mt-0.5 line-clamp-2 text-[0.68rem]">{hit.detail}</p>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

export function ScriptRiskPanel({ state }: ScriptRiskPanelProps) {
  const [settings, setSettings] = useState<Awaited<
    ReturnType<CompanionState["getSuspiciousActivitySettings"]>
  > | null>(null);
  const [consent, setConsent] = useState(false);
  const [error, setError] = useState<string | null>(null);
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

  const ensureEnabledThenScan = async () => {
    if (!consent) {
      setError("Tick the permission box, then run the scan.");
      return;
    }
    setScanning(true);
    setError(null);
    try {
      if (!enabled) {
        const next = await state.setSuspiciousActivitySettings({
          enabled: true,
          consentAck: true,
        });
        setSettings(next);
        setConsent(next.consent_ack);
      }
      const result = await state.runScriptRiskScan();
      setSnapshot(result);
    } catch (err: unknown) {
      setError(invokeErrorMessage(err, "Script risk scan failed."));
    } finally {
      setScanning(false);
    }
  };

  const hitCount =
    (snapshot?.persistence_hits.length ?? 0) +
    (snapshot?.mining_hits.length ?? 0) +
    (snapshot?.disk_hits?.length ?? 0);

  return (
    <section className="overflow-hidden rounded-2xl border border-border/60 bg-card/70">
      <div className="flex flex-wrap items-center justify-between gap-3 px-4 py-4 sm:px-5">
        <div className="flex min-w-0 items-center gap-3">
          <div className="bg-muted/80 text-foreground/80 flex size-10 shrink-0 items-center justify-center rounded-2xl">
            <FileSearch className="size-5" />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="text-base font-semibold tracking-tight">Script risk</h2>
              {snapshot?.scanned ? (
                <Badge variant={hitCount > 0 ? "caution" : "info"}>
                  {hitCount > 0 ? `${hitCount} found` : "Nothing found"}
                </Badge>
              ) : (
                <Badge variant="muted">Optional scan</Badge>
              )}
            </div>
            <p className="text-muted-foreground mt-0.5 text-xs leading-relaxed">
              Quick look for mining-like processes and risky scripts — not antivirus.
            </p>
          </div>
        </div>
        <Button size="default" disabled={scanning || !settings} onClick={() => void ensureEnabledThenScan()}>
          {scanning ? (
            <>
              <Loader2 className="size-4 animate-spin" />
              Scanning…
            </>
          ) : (
            "Run scan"
          )}
        </Button>
      </div>

      <div className="space-y-2.5 border-t border-border/40 bg-muted/10 px-4 py-3 sm:px-5">
        {error && <p className="text-destructive text-xs">{error}</p>}
        <label className="flex items-start gap-2 text-xs leading-relaxed">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={consent}
            onChange={(event) => {
              setConsent(event.target.checked);
              setError(null);
            }}
          />
          <span className="text-muted-foreground">
            Allow a local sample of startup entries, processes, and script files on this PC.
          </span>
        </label>
        <div className="flex items-center justify-between gap-3">
          <Label htmlFor="script-risk-enabled" className="text-muted-foreground text-xs">
            Also check during health checks
          </Label>
          <Switch
            id="script-risk-enabled"
            checked={enabled}
            disabled={!settings}
            onCheckedChange={(value) => {
              if (value && !consent) {
                setError("Tick the permission box first.");
                return;
              }
              void state
                .setSuspiciousActivitySettings({
                  enabled: value,
                  consentAck: value ? consent : settings?.consent_ack ?? false,
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
        <div className="border-t border-border/40 px-4 py-3.5 sm:px-5">
          <p className="text-muted-foreground text-xs leading-relaxed">
            No scan yet. Use <span className="text-foreground font-medium">Run scan</span> above
            after ticking permission — indicators only, not antivirus.
          </p>
        </div>
      ) : (
        <div className="space-y-3 border-t border-border/40 px-4 py-3.5 sm:px-5">
          <p className="text-sm leading-relaxed">{snapshot.summary}</p>
          <HitList title="Startup / script hosts" hits={snapshot.persistence_hits} />
          <HitList title="Mining-like processes" hits={snapshot.mining_hits} />
          <HitList title="Scripts on disk" hits={snapshot.disk_hits ?? []} />
          {hitCount === 0 && (
            <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
              No matches in this sample. That does not mean the PC is malware-free. Run scan again
              anytime from the button above.
            </p>
          )}
        </div>
      )}
    </section>
  );
}
