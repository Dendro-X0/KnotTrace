import type { ProtectAction, ProtectAlert } from "@/types";

export type ProtectReviewAssistKind = Extract<
  ProtectAction["kind"],
  "dns_assist" | "connect_assist"
>;

export interface ProtectReviewPlan {
  assistKinds: ProtectReviewAssistKind[];
  acknowledgeTitles: string[];
  actionableCount: number;
  indicatorCount: number;
}

export function alertFingerprint(alert: ProtectAlert): string {
  return `${alert.level}|${alert.title}|${alert.message.slice(0, 80)}`;
}

function usefulAssistKinds(alert: ProtectAlert): ProtectReviewAssistKind[] {
  const kinds: ProtectReviewAssistKind[] = [];
  for (const action of alert.actions) {
    if (action.kind === "dns_assist" || action.kind === "connect_assist") {
      if (!kinds.includes(action.kind)) kinds.push(action.kind);
    }
  }
  return kinds;
}

/**
 * Plan a one-click Protect review pass.
 * Always acknowledges active rows; collects assist hints from alert actions.
 * Runtime still attempts DNS/Connect when recommendations are eligible (user click = consent).
 */
export function planProtectReviewPass(
  alerts: ProtectAlert[],
  alreadyAcknowledged: ReadonlySet<string> = new Set(),
): ProtectReviewPlan {
  const assistKinds: ProtectReviewAssistKind[] = [];
  const acknowledgeTitles: string[] = [];
  let actionableCount = 0;
  let indicatorCount = 0;

  for (const alert of alerts) {
    const fp = alertFingerprint(alert);
    if (alreadyAcknowledged.has(fp)) continue;

    acknowledgeTitles.push(alert.title);
    const assists = usefulAssistKinds(alert);
    if (assists.length > 0) {
      actionableCount += 1;
      for (const kind of assists) {
        if (!assistKinds.includes(kind)) assistKinds.push(kind);
      }
    } else {
      indicatorCount += 1;
    }
  }

  return { assistKinds, acknowledgeTitles, actionableCount, indicatorCount };
}

export function filterActiveProtectAlerts(
  alerts: ProtectAlert[],
  acknowledged: ReadonlySet<string>,
): ProtectAlert[] {
  return alerts.filter((alert) => !acknowledged.has(alertFingerprint(alert)));
}

export function buildProtectReviewNote(input: {
  appliedDns: boolean;
  appliedConnect: boolean;
  dnsMessage?: string | null;
  connectMessage?: string | null;
  acknowledgedCount: number;
  skipped: string[];
}): string {
  const parts: string[] = [];
  if (input.appliedDns) {
    parts.push(input.dnsMessage?.trim() || "DNS assist applied");
  }
  if (input.appliedConnect) {
    parts.push(input.connectMessage?.trim() || "Connect assist applied");
  }
  if (input.acknowledgedCount > 0) {
    parts.push(
      `${input.acknowledgedCount} item${input.acknowledgedCount === 1 ? "" : "s"} marked reviewed`,
    );
  }
  if (input.skipped.length > 0) {
    parts.push(`Skipped: ${input.skipped.join("; ")}`);
  }
  if (parts.length === 0) {
    return "Nothing left to process.";
  }
  return parts.join(" · ");
}
