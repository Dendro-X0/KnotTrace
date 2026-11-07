import { isLocalPathOk, isTunnelActivePath } from "./pathTeaser.ts";

export type PageId =
  | "overview"
  | "dns"
  | "connect"
  | "protect"
  | "network"
  | "settings";

export interface NextStepAction {
  labelKey: string;
  page: PageId;
}

export interface NextStepPlan {
  titleKey: string;
  summaryKey: string;
  actions: NextStepAction[];
  prioritized: boolean;
}

export type SlowdownShape =
  | "page_start"
  | "under_load_lag"
  | "partial_site_failure"
  | "restricted_network"
  | "tunnel_overhead"
  | "link_local_issue"
  | "general_degradation";

export type DiagnosisConfidence = "high" | "medium" | "low";

export type CaptiveSignal = "confirmed" | "suspected" | "none";

export type BufferbloatGradeSignal = "moderate" | "severe" | "other";

/** Plain Quick layer for partial site / geo failures (P1). */
export type SiteFailureLayer =
  | "local"
  | "dns"
  | "active_path"
  | "destination_or_policy";

/** Narrow snapshot so ranking is unit-testable without Companion UI methods. */
export interface NextStepInput {
  shape: SlowdownShape;
  confidence: DiagnosisConfidence;
  thrashForbidden: boolean;
  dnsAssistEligible: boolean;
  connectAssistEligible: boolean;
  integrityCount: number;
  recurrenceCount: number;
  captiveState: CaptiveSignal;
  bufferbloatGrade: BufferbloatGradeSignal;
  activePath: string | null;
  /** Tunnel honesty: local_ok and break_hop none. */
  localOk: boolean;
  /** Raw path_break.local_ok for site/geo layering (ignores remote hops). */
  pathLocalOk: boolean;
  torSocksDown: boolean;
  breakHop: string | null;
  likelyProviderSide: boolean;
  proxyOnlyFailures: boolean;
  directAlsoFails: boolean;
  environmentClass: string | null;
}

/** Structural subset of CompanionState used by the planner. */
export interface NextStepStateSource {
  checkError?: string | null;
  report?: {
    score?: { grade?: string | null; summary?: string | null } | null;
    diagnosis?: {
      slowdown_shape: SlowdownShape;
      confidence: DiagnosisConfidence;
      summary?: string | null;
    } | null;
    upstream_pool?: { claim?: string | null } | null;
    integrity?: { indicator_count?: number | null } | null;
    path_break?: {
      break_hop?: string | null;
      local_ok?: boolean | null;
      active_path?: string | null;
      environment_class?: string | null;
    } | null;
    proxy_path_report?: {
      likely_provider_side?: boolean | null;
      proxy_only_failure_count?: number | null;
      direct_failure_count?: number | null;
    } | null;
    tunnel_compare?: {
      tor_detected?: boolean | null;
      tor_socks_reachable?: boolean | null;
    } | null;
    network_context?: {
      captive_portal?: { state?: string | null } | null;
    } | null;
    stability?: {
      bufferbloat?: { grade?: string | null } | null;
    } | null;
  } | null;
  trends: Array<{ slowdown_shape?: string | null }>;
  dnsRecommendation?: { should_apply: boolean } | null;
  connectRecommendation?: { should_apply: boolean } | null;
}

export type QuickHeroSummary =
  | { kind: "key"; key: string }
  | { kind: "text"; text: string };

const KEY = {
  softTitle: "nextSteps.copy.softTitle",
  softSummary: "nextSteps.copy.softSummary",
  bufferbloatTitle: "nextSteps.copy.bufferbloatTitle",
  bufferbloatSummary: "nextSteps.copy.bufferbloatSummary",
  captiveConfirmedTitle: "nextSteps.copy.captiveConfirmedTitle",
  captiveConfirmedSummary: "nextSteps.copy.captiveConfirmedSummary",
  captiveSuspectedTitle: "nextSteps.copy.captiveSuspectedTitle",
  captiveSuspectedSummary: "nextSteps.copy.captiveSuspectedSummary",
  pageStartTitle: "nextSteps.copy.pageStartTitle",
  pageStartSummary: "nextSteps.copy.pageStartSummary",
  partialThrashTitle: "nextSteps.copy.partialThrashTitle",
  partialThrashSummary: "nextSteps.copy.partialThrashSummary",
  partialTitle: "nextSteps.copy.partialTitle",
  partialSummary: "nextSteps.copy.partialSummary",
  partialLocalTitle: "nextSteps.copy.partialLocalTitle",
  partialLocalSummary: "nextSteps.copy.partialLocalSummary",
  partialDnsTitle: "nextSteps.copy.partialDnsTitle",
  partialDnsSummary: "nextSteps.copy.partialDnsSummary",
  partialDestinationTitle: "nextSteps.copy.partialDestinationTitle",
  partialDestinationSummary: "nextSteps.copy.partialDestinationSummary",
  restrictedTitle: "nextSteps.copy.restrictedTitle",
  restrictedSummary: "nextSteps.copy.restrictedSummary",
  tunnelThrashTitle: "nextSteps.copy.tunnelThrashTitle",
  tunnelThrashSummary: "nextSteps.copy.tunnelThrashSummary",
  tunnelTitle: "nextSteps.copy.tunnelTitle",
  tunnelSummary: "nextSteps.copy.tunnelSummary",
  tunnelHonestyTitle: "nextSteps.copy.tunnelHonestyTitle",
  tunnelHonestySummary: "nextSteps.copy.tunnelHonestySummary",
  tunnelLocalFirstTitle: "nextSteps.copy.tunnelLocalFirstTitle",
  tunnelLocalFirstSummary: "nextSteps.copy.tunnelLocalFirstSummary",
  tunnelBufferbloatTitle: "nextSteps.copy.tunnelBufferbloatTitle",
  tunnelBufferbloatSummary: "nextSteps.copy.tunnelBufferbloatSummary",
  torSocksDownTitle: "nextSteps.copy.torSocksDownTitle",
  torSocksDownSummary: "nextSteps.copy.torSocksDownSummary",
  linkLocalTitle: "nextSteps.copy.linkLocalTitle",
  linkLocalSummary: "nextSteps.copy.linkLocalSummary",
  actionNetwork: "nextSteps.action.network",
  actionProtect: "nextSteps.action.protect",
  actionDns: "nextSteps.action.dns",
  actionConnect: "nextSteps.action.connect",
  actionOverviewClaim: "nextSteps.action.overviewClaim",
  actionBenchmark: "nextSteps.action.benchmark",
  heroCaptiveConfirmed: "nextSteps.hero.captiveConfirmed",
  heroCaptiveSuspected: "nextSteps.hero.captiveSuspected",
  heroBufferbloat: "nextSteps.hero.bufferbloat",
  heroTunnelExpected: "nextSteps.hero.tunnelExpected",
  heroTunnelBufferbloat: "nextSteps.hero.tunnelBufferbloat",
  heroTorSocksDown: "nextSteps.hero.torSocksDown",
  heroPartialLocal: "nextSteps.hero.partialLocal",
  heroPartialDns: "nextSteps.hero.partialDns",
  heroPartialActivePath: "nextSteps.hero.partialActivePath",
  heroPartialDestination: "nextSteps.hero.partialDestination",
  heroCheckFailed: "nextSteps.hero.checkFailed",
  heroNoReport: "nextSteps.hero.noReport",
  heroGoodDefault: "nextSteps.hero.goodDefault",
  heroLookCloser: "nextSteps.hero.lookCloser",
} as const;

function confidenceBudget(confidence: DiagnosisConfidence): number {
  switch (confidence) {
    case "high":
      return 3;
    case "medium":
      return 2;
    case "low":
      return 1;
  }
}

function thrashForbiddenFromState(state: NextStepStateSource): boolean {
  const claim = state.report?.upstream_pool?.claim;
  return claim === "upstream_pool_poor" || claim === "active_path_recurring";
}

function recurrenceCountForShape(
  state: NextStepStateSource,
  shape: SlowdownShape | undefined,
): number {
  if (!shape || shape === "general_degradation") return 0;
  const recent = state.trends.slice(-24);
  return recent.filter((point) => point.slowdown_shape === shape).length;
}

function normalizeCaptiveState(state: string | null | undefined): CaptiveSignal {
  if (state === "confirmed") return "confirmed";
  if (state === "suspected") return "suspected";
  return "none";
}

function normalizeBufferbloatGrade(grade: string | null | undefined): BufferbloatGradeSignal {
  if (grade === "moderate" || grade === "severe") return grade;
  return "other";
}

export function isCaptiveSignal(state: CaptiveSignal): boolean {
  return state === "confirmed" || state === "suspected";
}

export function isPromotedBufferbloat(
  shape: SlowdownShape,
  grade: BufferbloatGradeSignal,
): boolean {
  return shape === "under_load_lag" || grade === "moderate" || grade === "severe";
}

/** Tunnel/proxy active (or tunnel_overhead shape) plus promoted bufferbloat — both factors. */
export function isTunnelBufferbloatOverlap(input: NextStepInput): boolean {
  if (input.torSocksDown) return false;
  if (!isPromotedBufferbloat(input.shape, input.bufferbloatGrade)) return false;
  return isTunnelActivePath(input.activePath) || input.shape === "tunnel_overhead";
}

/** Tunnel/proxy active and local stack OK — slower is expected, not a local repair job. */
export function isTunnelHonestyCase(input: NextStepInput): boolean {
  if (input.torSocksDown || input.thrashForbidden) return false;
  if (isTunnelBufferbloatOverlap(input)) return false;
  if (input.shape === "tunnel_overhead") {
    // Diagnosis says overhead: honesty unless we know local is broken on a tunnel path.
    return !isTunnelActivePath(input.activePath) || input.localOk;
  }
  return isTunnelActivePath(input.activePath) && input.localOk;
}

export function isTunnelLocalFirstCase(input: NextStepInput): boolean {
  if (input.torSocksDown) return false;
  return isTunnelActivePath(input.activePath) && !input.localOk;
}

/** P1: map partial site failure onto a plain Quick layer. */
export function classifySiteFailureLayer(input: NextStepInput): SiteFailureLayer | null {
  if (input.shape !== "partial_site_failure") return null;

  if (input.breakHop === "dns") return "dns";
  if (!input.pathLocalOk || input.breakHop === "local") return "local";

  if (
    input.thrashForbidden ||
    input.likelyProviderSide ||
    input.proxyOnlyFailures ||
    (isTunnelActivePath(input.activePath) && input.pathLocalOk)
  ) {
    return "active_path";
  }

  return "destination_or_policy";
}

function siteFailureCopy(input: NextStepInput): { titleKey: string; summaryKey: string } | null {
  const layer = classifySiteFailureLayer(input);
  if (!layer) return null;

  switch (layer) {
    case "local":
      return { titleKey: KEY.partialLocalTitle, summaryKey: KEY.partialLocalSummary };
    case "dns":
      return { titleKey: KEY.partialDnsTitle, summaryKey: KEY.partialDnsSummary };
    case "active_path":
      if (input.thrashForbidden) {
        return { titleKey: KEY.partialThrashTitle, summaryKey: KEY.partialThrashSummary };
      }
      return { titleKey: KEY.partialTitle, summaryKey: KEY.partialSummary };
    case "destination_or_policy":
      return {
        titleKey: KEY.partialDestinationTitle,
        summaryKey: KEY.partialDestinationSummary,
      };
  }
}

function siteFailureHeroKey(input: NextStepInput): string | null {
  const layer = classifySiteFailureLayer(input);
  if (!layer) return null;
  switch (layer) {
    case "local":
      return KEY.heroPartialLocal;
    case "dns":
      return KEY.heroPartialDns;
    case "active_path":
      return KEY.heroPartialActivePath;
    case "destination_or_policy":
      return KEY.heroPartialDestination;
  }
}

function pushSiteFailureActions(actions: NextStepAction[], input: NextStepInput) {
  const layer = classifySiteFailureLayer(input);
  if (!layer) return;

  const network: NextStepAction = { labelKey: KEY.actionNetwork, page: "network" };
  const protect: NextStepAction = { labelKey: KEY.actionProtect, page: "protect" };
  const dns: NextStepAction = { labelKey: KEY.actionDns, page: "dns" };
  const connect: NextStepAction = { labelKey: KEY.actionConnect, page: "connect" };
  const overviewClaim: NextStepAction = { labelKey: KEY.actionOverviewClaim, page: "overview" };

  switch (layer) {
    case "local":
      pushUnique(actions, network);
      if (input.integrityCount > 0) pushUnique(actions, protect);
      break;
    case "dns":
      if (input.dnsAssistEligible) {
        pushUnique(actions, dns);
        pushUnique(actions, network);
      } else {
        pushUnique(actions, network);
        pushUnique(actions, dns);
      }
      break;
    case "active_path":
      pushUnique(actions, network);
      if (input.connectAssistEligible) {
        pushUnique(actions, connect);
      } else if (input.thrashForbidden) {
        pushUnique(actions, overviewClaim);
      }
      break;
    case "destination_or_policy":
      pushUnique(actions, network);
      break;
  }
}

export function extractNextStepInput(state: NextStepStateSource): NextStepInput | null {
  const diagnosis = state.report?.diagnosis;
  if (!diagnosis) return null;

  const thrashForbidden = thrashForbiddenFromState(state);
  const path = state.report?.path_break;
  const tunnel = state.report?.tunnel_compare;
  const proxy = state.report?.proxy_path_report;
  return {
    shape: diagnosis.slowdown_shape,
    confidence: diagnosis.confidence,
    thrashForbidden,
    dnsAssistEligible: state.dnsRecommendation?.should_apply === true,
    connectAssistEligible:
      state.connectRecommendation?.should_apply === true && !thrashForbidden,
    integrityCount: state.report?.integrity?.indicator_count ?? 0,
    recurrenceCount: recurrenceCountForShape(state, diagnosis.slowdown_shape),
    captiveState: normalizeCaptiveState(state.report?.network_context?.captive_portal?.state),
    bufferbloatGrade: normalizeBufferbloatGrade(state.report?.stability?.bufferbloat?.grade),
    activePath: path?.active_path ?? null,
    localOk: isLocalPathOk(path),
    pathLocalOk: path?.local_ok === true,
    torSocksDown: Boolean(tunnel?.tor_detected && tunnel.tor_socks_reachable === false),
    breakHop: path?.break_hop ?? null,
    likelyProviderSide: proxy?.likely_provider_side === true,
    proxyOnlyFailures: (proxy?.proxy_only_failure_count ?? 0) > 0,
    directAlsoFails: (proxy?.direct_failure_count ?? 0) > 0,
    environmentClass: path?.environment_class ?? null,
  };
}

function shapeCopy(input: NextStepInput): { titleKey: string; summaryKey: string } {
  if (input.confidence === "low") {
    return { titleKey: KEY.softTitle, summaryKey: KEY.softSummary };
  }

  if (isCaptiveSignal(input.captiveState)) {
    if (input.captiveState === "confirmed") {
      return { titleKey: KEY.captiveConfirmedTitle, summaryKey: KEY.captiveConfirmedSummary };
    }
    return { titleKey: KEY.captiveSuspectedTitle, summaryKey: KEY.captiveSuspectedSummary };
  }

  if (isTunnelBufferbloatOverlap(input)) {
    return {
      titleKey: KEY.tunnelBufferbloatTitle,
      summaryKey: KEY.tunnelBufferbloatSummary,
    };
  }

  if (isPromotedBufferbloat(input.shape, input.bufferbloatGrade)) {
    return { titleKey: KEY.bufferbloatTitle, summaryKey: KEY.bufferbloatSummary };
  }

  const siteFailure = siteFailureCopy(input);
  if (siteFailure) return siteFailure;

  if (input.torSocksDown) {
    return { titleKey: KEY.torSocksDownTitle, summaryKey: KEY.torSocksDownSummary };
  }

  if (isTunnelLocalFirstCase(input)) {
    return { titleKey: KEY.tunnelLocalFirstTitle, summaryKey: KEY.tunnelLocalFirstSummary };
  }

  if (isTunnelHonestyCase(input)) {
    return { titleKey: KEY.tunnelHonestyTitle, summaryKey: KEY.tunnelHonestySummary };
  }

  if (input.shape === "general_degradation") {
    return { titleKey: KEY.softTitle, summaryKey: KEY.softSummary };
  }

  switch (input.shape) {
    case "page_start":
      return { titleKey: KEY.pageStartTitle, summaryKey: KEY.pageStartSummary };
    case "under_load_lag":
      return { titleKey: KEY.bufferbloatTitle, summaryKey: KEY.bufferbloatSummary };
    case "partial_site_failure":
      // Defensive: classifier should have returned above.
      return { titleKey: KEY.partialTitle, summaryKey: KEY.partialSummary };
    case "restricted_network":
      return { titleKey: KEY.restrictedTitle, summaryKey: KEY.restrictedSummary };
    case "tunnel_overhead":
      if (input.thrashForbidden) {
        return { titleKey: KEY.tunnelThrashTitle, summaryKey: KEY.tunnelThrashSummary };
      }
      return { titleKey: KEY.tunnelHonestyTitle, summaryKey: KEY.tunnelHonestySummary };
    case "link_local_issue":
      return { titleKey: KEY.linkLocalTitle, summaryKey: KEY.linkLocalSummary };
    default:
      return { titleKey: KEY.softTitle, summaryKey: KEY.softSummary };
  }
}

function pushUnique(actions: NextStepAction[], action: NextStepAction) {
  if (actions.some((item) => item.page === action.page && item.labelKey === action.labelKey)) {
    return;
  }
  actions.push(action);
}

function candidatePool(input: NextStepInput): NextStepAction[] {
  const actions: NextStepAction[] = [];
  const network: NextStepAction = { labelKey: KEY.actionNetwork, page: "network" };
  const protect: NextStepAction = { labelKey: KEY.actionProtect, page: "protect" };
  const dns: NextStepAction = { labelKey: KEY.actionDns, page: "dns" };
  const overviewClaim: NextStepAction = { labelKey: KEY.actionOverviewClaim, page: "overview" };
  const overviewBenchmark: NextStepAction = { labelKey: KEY.actionBenchmark, page: "overview" };

  const captive = isCaptiveSignal(input.captiveState);

  if (!captive && isTunnelBufferbloatOverlap(input)) {
    pushUnique(actions, network);
    pushUnique(actions, overviewBenchmark);
    return actions;
  }

  switch (input.shape) {
    case "page_start":
      pushUnique(actions, dns);
      pushUnique(actions, network);
      if (input.integrityCount > 0) pushUnique(actions, protect);
      break;
    case "under_load_lag":
      pushUnique(actions, network);
      pushUnique(actions, overviewBenchmark);
      break;
    case "partial_site_failure":
      pushSiteFailureActions(actions, input);
      break;
    case "restricted_network":
      if (captive) {
        pushUnique(actions, network);
      } else {
        pushUnique(actions, protect);
        pushUnique(actions, network);
      }
      break;
    case "tunnel_overhead":
      pushUnique(actions, network);
      if (input.thrashForbidden) {
        pushUnique(actions, overviewClaim);
      }
      // Honesty / SOCKS-down / local-first: Network only — no Connect speed-fix CTA.
      break;
    case "link_local_issue":
      pushUnique(actions, network);
      if (input.integrityCount > 0) pushUnique(actions, protect);
      break;
    case "general_degradation":
    default:
      if (isPromotedBufferbloat(input.shape, input.bufferbloatGrade)) {
        pushUnique(actions, network);
        pushUnique(actions, overviewBenchmark);
      } else {
        pushUnique(actions, network);
      }
      break;
  }

  const siteLayer = classifySiteFailureLayer(input);
  if (
    !siteLayer &&
    !isTunnelBufferbloatOverlap(input) &&
    (input.torSocksDown || isTunnelHonestyCase(input) || isTunnelLocalFirstCase(input))
  ) {
    return [network];
  }

  if (
    !captive &&
    input.shape !== "page_start" &&
    !siteLayer &&
    !isTunnelBufferbloatOverlap(input) &&
    input.dnsAssistEligible
  ) {
    pushUnique(actions, dns);
  } else if (siteLayer === "local" && input.dnsAssistEligible) {
    pushUnique(actions, dns);
  }

  if (actions.length === 0) {
    pushUnique(actions, network);
  }

  return actions;
}

function preferredPageForRecurrence(input: NextStepInput): PageId | null {
  switch (input.shape) {
    case "page_start":
      return "dns";
    case "partial_site_failure": {
      const layer = classifySiteFailureLayer(input);
      if (layer === "dns") return "dns";
      if (layer === "active_path" && input.connectAssistEligible) return "connect";
      return "network";
    }
    case "restricted_network":
      return isCaptiveSignal(input.captiveState) ? "network" : "protect";
    case "under_load_lag":
      return "network";
    case "tunnel_overhead":
      return "network";
    case "link_local_issue":
      return "network";
    default:
      return null;
  }
}

function applyRecurrencePriority(
  actions: NextStepAction[],
  input: NextStepInput,
): { actions: NextStepAction[]; prioritized: boolean } {
  if (input.recurrenceCount < 2) return { actions, prioritized: false };

  const preferred = preferredPageForRecurrence(input);
  if (!preferred) return { actions, prioritized: false };

  const copy = [...actions];
  const idx = copy.findIndex((action) => action.page === preferred);
  if (idx <= 0) return { actions: copy, prioritized: false };

  const [target] = copy.splice(idx, 1);
  copy.unshift(target);
  return { actions: copy, prioritized: true };
}

/** Pure ranking entry — used by UI and unit tests. */
export function buildNextStepPlanFromInput(input: NextStepInput): NextStepPlan {
  const { titleKey, summaryKey } = shapeCopy(input);
  const budget = confidenceBudget(input.confidence);
  const truncated = candidatePool(input).slice(0, budget);
  const ranked = applyRecurrencePriority(truncated, input);
  return {
    titleKey,
    summaryKey,
    actions: ranked.actions,
    prioritized: ranked.prioritized,
  };
}

export function buildNextStepPlan(state: NextStepStateSource): NextStepPlan | null {
  const input = extractNextStepInput(state);
  if (!input) return null;
  return buildNextStepPlanFromInput(input);
}

/** Plain-language Quick hero — captive / bufferbloat beat score summary. */
export function buildQuickHeroSummary(state: NextStepStateSource): QuickHeroSummary {
  if (state.checkError) {
    return { kind: "key", key: KEY.heroCheckFailed };
  }
  const report = state.report;
  if (!report) {
    return { kind: "key", key: KEY.heroNoReport };
  }

  const captive = normalizeCaptiveState(report.network_context?.captive_portal?.state);
  if (captive === "confirmed") return { kind: "key", key: KEY.heroCaptiveConfirmed };
  if (captive === "suspected") return { kind: "key", key: KEY.heroCaptiveSuspected };

  const shape = report.diagnosis?.slowdown_shape;
  const grade = normalizeBufferbloatGrade(report.stability?.bufferbloat?.grade);
  const overlapInput = extractNextStepInput(state);
  if (overlapInput && isTunnelBufferbloatOverlap(overlapInput)) {
    return { kind: "key", key: KEY.heroTunnelBufferbloat };
  }
  if ((shape && isPromotedBufferbloat(shape, grade)) || grade !== "other") {
    return { kind: "key", key: KEY.heroBufferbloat };
  }

  if (shape === "partial_site_failure") {
    const input = extractNextStepInput(state);
    if (input) {
      const heroKey = siteFailureHeroKey(input);
      if (heroKey) return { kind: "key", key: heroKey };
    }
  }

  const tunnel = report.tunnel_compare;
  const path = report.path_break;
  if (tunnel?.tor_detected && tunnel.tor_socks_reachable === false) {
    return { kind: "key", key: KEY.heroTorSocksDown };
  }
  if (
    isTunnelActivePath(path?.active_path) &&
    isLocalPathOk(path)
  ) {
    return { kind: "key", key: KEY.heroTunnelExpected };
  }
  if (shape === "tunnel_overhead") {
    return { kind: "key", key: KEY.heroTunnelExpected };
  }

  // High-traffic shaped diagnoses: prefer localized next-step summary over English Rust text.
  if (shape && shape !== "general_degradation") {
    const plan = buildNextStepPlan(state);
    if (plan?.summaryKey) {
      return { kind: "key", key: plan.summaryKey };
    }
  }

  const scoreGrade = report.score?.grade;
  const scoreLine = report.score?.summary?.trim();
  if (scoreGrade === "good") {
    return scoreLine
      ? { kind: "text", text: scoreLine }
      : { kind: "key", key: KEY.heroGoodDefault };
  }
  if (scoreLine) return { kind: "text", text: scoreLine };
  const diagnosis = report.diagnosis?.summary?.trim();
  if (diagnosis) return { kind: "text", text: diagnosis };
  return { kind: "key", key: KEY.heroLookCloser };
}

/** Whether Overview Quick should show next-step CTAs in the hero. */
export function shouldSurfaceQuickNextCtas(
  state: NextStepStateSource,
  plan: NextStepPlan | null,
): boolean {
  if (!plan) return false;
  const report = state.report;
  if (!report) return false;

  const grade = report.score?.grade;
  const integrityCount = report.integrity?.indicator_count ?? 0;
  const pathBroken = Boolean(report.path_break && report.path_break.break_hop !== "none");
  const captive = normalizeCaptiveState(report.network_context?.captive_portal?.state);
  const bufferbloat = normalizeBufferbloatGrade(report.stability?.bufferbloat?.grade);
  const shape = report.diagnosis?.slowdown_shape;

  if (grade !== "good") return true;
  if (integrityCount > 0 || pathBroken) return true;
  if (isCaptiveSignal(captive)) return true;
  if (shape && isPromotedBufferbloat(shape, bufferbloat)) return true;
  if (bufferbloat !== "other") return true;
  return false;
}
