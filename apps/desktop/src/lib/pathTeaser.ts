export type PathBreakPathId = "direct" | "system_proxy" | "vpn_tunnel" | "tor_socks" | string;

export interface PathTeaserPathBreak {
  title?: string | null;
  summary?: string | null;
  active_path?: PathBreakPathId | null;
  local_ok?: boolean | null;
  break_hop?: string | null;
}

export interface PathTeaserTunnelCompare {
  expectation?: string | null;
  tor_detected?: boolean | null;
  tor_socks_reachable?: boolean | null;
  vpn_detected?: boolean | null;
  proxy_enabled?: boolean | null;
}

export type PathExpectation =
  | { kind: "key"; key: string }
  | { kind: "text"; text: string }
  | null;

export interface QuickPathTeaserContent {
  title: string;
  summary: string;
  expectation: PathExpectation;
  tunnelActive: boolean;
}

export function isTunnelActivePath(path: PathBreakPathId | null | undefined): boolean {
  return path === "vpn_tunnel" || path === "tor_socks" || path === "system_proxy";
}

export function isLocalPathOk(
  path: PathTeaserPathBreak | null | undefined,
): boolean {
  if (!path) return false;
  return path.local_ok === true && (path.break_hop == null || path.break_hop === "none");
}

function fallbackExpectationKey(
  path: PathBreakPathId | null | undefined,
  tunnel: PathTeaserTunnelCompare | null | undefined,
): string {
  if (tunnel?.tor_detected && tunnel.tor_socks_reachable === false) {
    return "nextSteps.path.expectationTorSocksDown";
  }
  if (path === "tor_socks" || tunnel?.tor_detected) {
    return "nextSteps.path.expectationFallbackTor";
  }
  if (path === "vpn_tunnel" || tunnel?.vpn_detected) {
    return "nextSteps.path.expectationFallbackVpn";
  }
  return "nextSteps.path.expectationFallbackProxy";
}

/** Pure Path teaser content for Overview Quick (and unit tests). */
export function buildQuickPathTeaserContent(
  path: PathTeaserPathBreak | null | undefined,
  tunnel?: PathTeaserTunnelCompare | null,
): QuickPathTeaserContent {
  const title = path?.title?.trim() || "Path not classified yet";
  const summary =
    path?.summary?.trim() ||
    "Run a health check to see whether the issue looks local or on the path.";
  const active = path?.active_path ?? null;
  const tunnelActive = isTunnelActivePath(active);

  if (!tunnelActive) {
    return { title, summary, expectation: null, tunnelActive: false };
  }

  const localOk = isLocalPathOk(path);
  const socksDown = Boolean(tunnel?.tor_detected && tunnel.tor_socks_reachable === false);
  const backendExpectation = tunnel?.expectation?.trim();

  if (!localOk) {
    return {
      title,
      summary,
      expectation: { kind: "key", key: "nextSteps.path.expectationLocalFirst" },
      tunnelActive: true,
    };
  }

  if (socksDown) {
    return {
      title,
      summary,
      expectation: backendExpectation
        ? { kind: "text", text: backendExpectation }
        : { kind: "key", key: "nextSteps.path.expectationTorSocksDown" },
      tunnelActive: true,
    };
  }

  if (backendExpectation && !/no encrypted tunnel expectation/i.test(backendExpectation)) {
    return {
      title,
      summary,
      expectation: { kind: "text", text: backendExpectation },
      tunnelActive: true,
    };
  }

  return {
    title,
    summary,
    expectation: { kind: "key", key: fallbackExpectationKey(active, tunnel) },
    tunnelActive: true,
  };
}
