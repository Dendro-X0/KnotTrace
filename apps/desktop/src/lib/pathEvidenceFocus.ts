import { isTunnelActivePath } from "./pathTeaser.ts";

export type PathEvidenceFocusId =
  | "path_break"
  | "proxy_path"
  | "upstream_pool"
  | "tunnel_compare"
  | "egress_timeline";

export interface PathEvidenceFocusInput {
  activePath?: string | null;
  breakHop?: string | null;
  proxyEnabled?: boolean | null;
  proxyOnlyFailureCount?: number | null;
  likelyProviderSide?: boolean | null;
  upstreamClaim?: string | null;
  tunnelVpnDetected?: boolean | null;
  tunnelTorDetected?: boolean | null;
  tunnelProxyEnabled?: boolean | null;
  egressClaim?: string | null;
}

export interface PathEvidenceFocus {
  id: PathEvidenceFocusId;
  reasonKey: string;
}

const REASON = {
  tunnel: "pathFocus.reason.tunnel",
  upstream: "pathFocus.reason.upstream",
  proxy: "pathFocus.reason.proxy",
  pathBreak: "pathFocus.reason.pathBreak",
  egress: "pathFocus.reason.egress",
  default: "pathFocus.reason.default",
} as const;

const UPSTREAM_FOCUS = new Set([
  "active_path_impaired",
  "active_path_recurring",
  "upstream_pool_poor",
]);

const EGRESS_FOCUS = new Set(["ip_changed", "churning", "unstable"]);

/** Pick one Advanced Path panel to open first. */
export function recommendPathEvidenceFocus(
  input: PathEvidenceFocusInput,
): PathEvidenceFocus {
  const tunnelSignals =
    isTunnelActivePath(input.activePath) ||
    input.tunnelVpnDetected === true ||
    input.tunnelTorDetected === true ||
    input.tunnelProxyEnabled === true;

  if (tunnelSignals) {
    return { id: "tunnel_compare", reasonKey: REASON.tunnel };
  }

  if (input.upstreamClaim && UPSTREAM_FOCUS.has(input.upstreamClaim)) {
    return { id: "upstream_pool", reasonKey: REASON.upstream };
  }

  if (
    input.proxyEnabled &&
    ((input.proxyOnlyFailureCount ?? 0) > 0 || input.likelyProviderSide === true)
  ) {
    return { id: "proxy_path", reasonKey: REASON.proxy };
  }

  if (input.breakHop && input.breakHop !== "none") {
    return { id: "path_break", reasonKey: REASON.pathBreak };
  }

  if (input.egressClaim && EGRESS_FOCUS.has(input.egressClaim)) {
    return { id: "egress_timeline", reasonKey: REASON.egress };
  }

  return { id: "path_break", reasonKey: REASON.default };
}

export function pathEvidenceFocusFromReport(report: {
  path_break?: {
    active_path?: string | null;
    break_hop?: string | null;
  } | null;
  proxy_path_report?: {
    proxy_only_failure_count?: number | null;
    likely_provider_side?: boolean | null;
  } | null;
  upstream_pool?: { claim?: string | null } | null;
  tunnel_compare?: {
    vpn_detected?: boolean | null;
    tor_detected?: boolean | null;
    proxy_enabled?: boolean | null;
  } | null;
  egress_timeline?: { claim?: string | null } | null;
  environment?: { proxy?: { enabled?: boolean | null } | null } | null;
} | null | undefined): PathEvidenceFocus {
  return recommendPathEvidenceFocus({
    activePath: report?.path_break?.active_path ?? null,
    breakHop: report?.path_break?.break_hop ?? null,
    proxyEnabled: report?.environment?.proxy?.enabled ?? null,
    proxyOnlyFailureCount: report?.proxy_path_report?.proxy_only_failure_count ?? null,
    likelyProviderSide: report?.proxy_path_report?.likely_provider_side ?? null,
    upstreamClaim: report?.upstream_pool?.claim ?? null,
    tunnelVpnDetected: report?.tunnel_compare?.vpn_detected ?? null,
    tunnelTorDetected: report?.tunnel_compare?.tor_detected ?? null,
    tunnelProxyEnabled: report?.tunnel_compare?.proxy_enabled ?? null,
    egressClaim: report?.egress_timeline?.claim ?? null,
  });
}
