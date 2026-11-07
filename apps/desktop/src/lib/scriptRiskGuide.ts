import type { SuspiciousActivityHit, SuspiciousActivitySnapshot } from "../types.ts";

export type ScriptRiskHitKind = "persistence" | "mining" | "disk";

export interface ScriptRiskGuideStep {
  id: string;
  title: string;
  detail: string;
  /** Optional clipboard payload for “Copy …” actions. */
  copyText?: string;
  /** Open OS startup settings when true (Windows ms-settings URI). */
  openStartupSettings?: boolean;
}

export interface ScriptRiskGuide {
  headline: string;
  steps: ScriptRiskGuideStep[];
  honesty: string;
}

function looksLikeFilesystemPath(location: string): boolean {
  const trimmed = location.trim();
  if (!trimmed) return false;
  if (/^hk(cu|lm|cr|u|cc)/i.test(trimmed)) return false;
  if (/^[a-z]:[/\\]/i.test(trimmed)) return true;
  if (trimmed.startsWith("\\\\") || trimmed.startsWith("/")) return true;
  return /[/\\]/.test(trimmed) && trimmed.includes(".");
}

/** Prefer revealing a concrete file path when the hit location looks local. */
export function revealablePathForHit(hit: SuspiciousActivityHit): string | null {
  if (looksLikeFilesystemPath(hit.location)) return hit.location.trim();
  if (looksLikeFilesystemPath(hit.name)) return hit.name.trim();
  return null;
}

export function flattenScriptRiskHits(snapshot: SuspiciousActivitySnapshot | null | undefined): Array<{
  kind: ScriptRiskHitKind;
  hit: SuspiciousActivityHit;
}> {
  if (!snapshot?.scanned) return [];
  return [
    ...snapshot.persistence_hits.map((hit) => ({ kind: "persistence" as const, hit })),
    ...snapshot.mining_hits.map((hit) => ({ kind: "mining" as const, hit })),
    ...(snapshot.disk_hits ?? []).map((hit) => ({ kind: "disk" as const, hit })),
  ];
}

/** Pure next-step guidance from a script-risk snapshot (unit-tested). */
export function buildScriptRiskGuide(
  snapshot: SuspiciousActivitySnapshot | null | undefined,
): ScriptRiskGuide | null {
  if (!snapshot?.scanned) return null;

  const persistence = snapshot.persistence_hits.length;
  const mining = snapshot.mining_hits.length;
  const disk = snapshot.disk_hits?.length ?? 0;
  const total = persistence + mining + disk;
  const steps: ScriptRiskGuideStep[] = [];

  if (total === 0) {
    return {
      headline: "No indicators in this bounded sample",
      steps: [
        {
          id: "empty-rescan",
          title: "Scan again after installing software",
          detail:
            "This sample only checks startup entries, a process snapshot, and a few user folders. Empty does not mean the PC is malware-free.",
        },
        {
          id: "empty-health",
          title: "Keep using health checks for path truth",
          detail:
            "Script risk is local heuristics. Path, DNS, and Protect alerts still matter for connection exposure.",
        },
      ],
      honesty:
        "KnotTrace shows indicators only — it does not quarantine, kill processes, or declare the device clean.",
    };
  }

  if (disk > 0) {
    const first = snapshot.disk_hits?.[0];
    steps.push({
      id: "disk-review",
      title: "Review script files in Downloads / Temp",
      detail:
        "Open the folder for each hit and delete only files you recognize as unwanted. KnotTrace will not remove them for you.",
      copyText: first ? revealablePathForHit(first) ?? first.location : undefined,
    });
  }

  if (mining > 0) {
    const first = snapshot.mining_hits[0];
    steps.push({
      id: "mining-taskmgr",
      title: "Check mining-like processes in Task Manager",
      detail:
        "Press Ctrl+Shift+Esc, find the process name, and end it only if you are sure it is unwanted. High CPU alone is not proof of malware.",
      copyText: first?.name,
    });
  }

  if (persistence > 0) {
    const first = snapshot.persistence_hits[0];
    steps.push({
      id: "persistence-startup",
      title: "Review Startup apps and Run keys",
      detail:
        "Open Windows Startup settings (or your OS equivalent) and disable entries you did not install. Copy the location below if you need to inspect it.",
      copyText: first?.location ?? first?.name,
      openStartupSettings: true,
    });
  }

  return {
    headline:
      total === 1
        ? "1 indicator to review"
        : `${total} indicators to review`,
    steps: steps.slice(0, 3),
    honesty:
      "These are heuristics, not a malware verdict. KnotTrace does not clean or quarantine — you decide what to change.",
  };
}
