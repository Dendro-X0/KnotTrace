export type HealthGrade = "good" | "fair" | "poor";

export type PageId = "overview" | "dns" | "connect" | "protect" | "network";

export interface HealthScore {
  grade: HealthGrade;
  score: number;
  summary: string;
  reasons: string[];
}

export interface LatencySample {
  target: string;
  avg_ms: number;
  loss_pct: number;
}

export interface DnsProbe {
  resolver: string;
  query: string;
  latency_ms: number;
  success: boolean;
}

export interface ProbeResult {
  gateway: LatencySample | null;
  internet: LatencySample | null;
  dns: DnsProbe[];
  duration_ms: number;
}

export interface EnvironmentSnapshot {
  hostname: string;
  tags: string[];
  default_gateway: string | null;
  active_interface: string | null;
  proxy: {
    enabled: boolean;
    server: string | null;
    source: string;
  };
  dns_servers: { address: string }[];
  tor?: TorStatus;
}

export interface HealthReport {
  timestamp: string;
  environment: EnvironmentSnapshot;
  probe: ProbeResult;
  score: HealthScore;
  dns_integrity?: DnsIntegrityStatus | null;
  diagnosis?: NetworkDiagnosis | null;
  stability?: StabilityProbeResult | null;
  site_reachability?: SiteReachabilityStatus | null;
  proxy_path_report?: ProxyPathReport | null;
  egress?: EgressReport | null;
  network_context?: NetworkContextReport | null;
  recommendations?: NetworkRecommendations | null;
}

export type EgressConfidence = "high" | "medium" | "low" | "unknown";
export type NetworkContextKind =
  | "home_lan"
  | "guest_wifi"
  | "public_cellular"
  | "captive_portal"
  | "unknown";
export type NetworkRiskLevel = "low" | "moderate" | "high";
export type CaptivePortalState = "not_detected" | "suspected" | "confirmed";

export interface EgressEndpointResult {
  provider: string;
  ip: string | null;
  latency_ms: number | null;
  success: boolean;
  error: string | null;
}

export interface EgressPathReport {
  kind: "system" | "tor_socks";
  primary_ip: string | null;
  endpoints: EgressEndpointResult[];
  summary: string;
}

export interface EgressReport {
  primary_ip: string | null;
  confidence: EgressConfidence;
  system_path: EgressPathReport;
  tor_path?: EgressPathReport | null;
  summary: string;
}

export interface CaptivePortalStatus {
  state: CaptivePortalState;
  probe_url: string;
  status_code: number | null;
  redirected: boolean;
  summary: string;
}

export interface NetworkContextReport {
  kind: NetworkContextKind;
  risk_level: NetworkRiskLevel;
  captive_portal: CaptivePortalStatus;
  signals: string[];
  summary: string;
}

export interface NetworkRecommendation {
  category: string;
  priority: number;
  title: string;
  message: string;
}

export interface NetworkRecommendations {
  items: NetworkRecommendation[];
  summary: string;
}

export type SiteReachErrorKind =
  | "timeout"
  | "connection_reset"
  | "tls"
  | "http_server"
  | "http_blocked"
  | "proxy"
  | "dns"
  | "unknown";

export interface SiteReachResult {
  domain: string;
  success: boolean;
  status_code: number | null;
  latency_ms: number | null;
  error: string | null;
  error_kind?: SiteReachErrorKind | null;
}

export interface SiteReachabilityStatus {
  checked_domains: number;
  success_count: number;
  failure_count: number;
  results: SiteReachResult[];
  summary: string;
}

export type ProxyPathConfidence = "high" | "medium" | "low";

export interface ProxyPathDomainComparison {
  domain: string;
  proxy: SiteReachResult;
  direct: SiteReachResult;
  proxy_only_failure: boolean;
}

export interface ProxyPathReport {
  comparisons: ProxyPathDomainComparison[];
  checked_domains: number;
  proxy_failure_count: number;
  direct_failure_count: number;
  proxy_only_failure_count: number;
  confidence: ProxyPathConfidence;
  likely_provider_side: boolean;
  summary: string;
}

export type BufferbloatGrade = "none" | "mild" | "moderate" | "severe";

export interface BufferbloatProbe {
  idle_latency_ms: number;
  loaded_latency_ms: number;
  latency_delta_ms: number;
  grade: BufferbloatGrade;
  summary: string;
}

export interface MtuProbe {
  estimated_path_mtu: number | null;
  recommended_tcp_mss: number | null;
  fragmentation_risk: boolean;
  summary: string;
}

export interface StabilityProbeResult {
  bufferbloat: BufferbloatProbe | null;
  mtu: MtuProbe | null;
  duration_ms: number;
}

export interface ThroughputSettings {
  download_bytes: number;
  upload_bytes: number;
}

export interface ThroughputProbeResult {
  download_mbps: number | null;
  upload_mbps: number | null;
  download_bytes: number;
  upload_bytes: number;
  duration_ms: number;
  summary: string;
}

export interface ProxyPathSample {
  proxy_name: string;
  delay_ms: number | null;
  is_current: boolean;
}

export interface ProxyPathComparison {
  group_name: string;
  test_url: string;
  samples: ProxyPathSample[];
  fastest_proxy: string | null;
  summary: string;
}

export interface TorStatus {
  detected: boolean;
  socks_endpoint: string | null;
  socks_reachable: boolean;
  source: string;
}

export type DnsIntegrityState = "ok" | "caution" | "suspicious";
export type DnsIntegrityConfidence = "low" | "medium" | "high";

export interface DnsIntegrityFinding {
  domain: string;
  local_answers: string[];
  trusted_answers: string[];
  local_error: string | null;
  trusted_error_count: number;
  reason: string;
}

export interface DnsIntegrityStatus {
  state: DnsIntegrityState;
  confidence: DnsIntegrityConfidence;
  mismatch_count: number;
  checked_domains: number;
  summary: string;
  details: DnsIntegrityFinding[];
}

export interface DnsIntegritySettings {
  verification_domains: string[];
}

export type BottleneckCategory =
  | "internet_unreachable"
  | "internet_loss"
  | "internet_latency"
  | "gateway"
  | "dns_failure"
  | "dns_slow"
  | "dns_integrity"
  | "site_access"
  | "proxy_path"
  | "vpn_tunnel"
  | "tor_tunnel"
  | "bufferbloat"
  | "mtu_fragmentation"
  | "wifi_path"
  | "cellular_path"
  | "public_network"
  | "captive_portal"
  | "egress_unstable"
  | "healthy";

export interface BottleneckHint {
  category: BottleneckCategory;
  severity: "info" | "warning" | "critical";
  title: string;
  message: string;
  suggestions: string[];
}

export interface NetworkDiagnosis {
  summary: string;
  primary_bottleneck: BottleneckCategory | null;
  slowdown_shape:
    | "page_start"
    | "under_load_lag"
    | "partial_site_failure"
    | "restricted_network"
    | "tunnel_overhead"
    | "link_local_issue"
    | "general_degradation";
  confidence: "high" | "medium" | "low";
  hints: BottleneckHint[];
}

export interface ExternalSpeedtestNote {
  download_mbps: number | null;
  upload_mbps: number | null;
  ping_ms: number | null;
  source: string;
}

export interface BenchmarkProbeSummary {
  internet_latency_ms: number | null;
  internet_loss_pct: number | null;
  gateway_latency_ms: number | null;
  dns_latency_ms: number | null;
}

export interface BenchmarkSnapshot {
  id: string;
  timestamp: string;
  label: string;
  environment_fingerprint: string;
  health_score: number;
  health_grade: HealthGrade;
  probe_summary: BenchmarkProbeSummary;
  dns_integrity_state: DnsIntegrityState | null;
  primary_bottleneck: BottleneckCategory | null;
  slowdown_shape?:
    | "page_start"
    | "under_load_lag"
    | "partial_site_failure"
    | "restricted_network"
    | "tunnel_overhead"
    | "link_local_issue"
    | "general_degradation"
    | null;
  external_speedtest?: ExternalSpeedtestNote | null;
  notes?: string | null;
}

export interface MonitorStatus {
  enabled: boolean;
  last_reason: string;
  poll_interval_secs: number;
  recheck_interval_secs: number;
}

export interface DnsCandidateResult {
  resolver: string;
  label: string;
  latency_ms: number;
  success: boolean;
}

export interface DnsAssistRecommendation {
  current_latency_ms: number | null;
  current_resolvers: string[];
  candidates: DnsCandidateResult[];
  recommended: DnsCandidateResult | null;
  should_apply: boolean;
  reason: string;
  interface_alias: string | null;
}

export interface DnsBackup {
  interface_alias: string;
  previous_servers: string[];
  use_dhcp: boolean;
  applied_resolver: string;
  applied_at: string;
}

export interface DnsAssistState {
  active: boolean;
  backup: DnsBackup | null;
  can_apply: boolean;
  platform_note: string;
}

export interface ConnectConfig {
  api_base: string;
  secret: string | null;
  auto_discovered: boolean;
}

export interface ProxyKernelInfo {
  kind: string;
  api_base: string;
  version: string | null;
}

export interface ProxyMemberView {
  name: string;
  proxy_type: string;
  delay_ms: number | null;
  alive: boolean;
}

export interface ProxyGroupView {
  name: string;
  group_type: string;
  current: string | null;
  members: ProxyMemberView[];
}

export interface ProxySwitchRecommendation {
  group_name: string;
  current_proxy: string | null;
  target_proxy: string;
  current_delay_ms: number | null;
  target_delay_ms: number | null;
}

export interface ConnectRecommendation {
  kernel: ProxyKernelInfo | null;
  groups: ProxyGroupView[];
  recommended_switch: ProxySwitchRecommendation | null;
  should_apply: boolean;
  reason: string;
}

export interface ProtectAction {
  kind: "dns_assist" | "connect_assist" | "run_check";
  label: string;
}

export interface ProtectAlert {
  level: "info" | "warning" | "critical";
  title: string;
  message: string;
  actions: ProtectAction[];
}

export interface ProtectSettings {
  enabled: boolean;
  notify_on_grade_drop: boolean;
  notify_on_untrusted_network: boolean;
  notify_on_degraded: boolean;
  auto_apply_dns: boolean;
  auto_apply_connect: boolean;
  auto_apply_on_untrusted_only: boolean;
  auto_recover_dns_integrity: boolean;
  auto_recover_site_access: boolean;
}

export interface AutoProtectResult {
  applied: Array<{ kind: string; message: string; success: boolean }>;
  skipped_reason: string | null;
}

export interface ProtectStatus {
  trust_level: "trusted" | "caution" | "untrusted";
  alerts: ProtectAlert[];
  settings: ProtectSettings;
  summary: string;
}

export interface HistoryTrendPoint {
  timestamp: string;
  score: number;
  grade: HealthGrade;
  dns_latency_ms: number | null;
  internet_latency_ms: number | null;
  gateway_latency_ms: number | null;
  dns_integrity_state?: DnsIntegrityState | null;
  dns_integrity_confidence?: DnsIntegrityConfidence | null;
  dns_integrity_mismatch_count?: number | null;
  slowdown_shape?:
    | "page_start"
    | "under_load_lag"
    | "partial_site_failure"
    | "restricted_network"
    | "tunnel_overhead"
    | "link_local_issue"
    | "general_degradation"
    | null;
}

export interface UpdateCheck {
  available: boolean;
  current_version: string;
  latest_version: string | null;
  release_url: string | null;
  notes: string | null;
  can_install_in_app: boolean;
}

export interface UpdateProgress {
  phase: string;
  downloaded: number;
  total: number | null;
}

export const PAGE_STORAGE_KEY = "knottrace-active-page";

export const PAGE_TITLES: Record<PageId, string> = {
  overview: "Overview",
  dns: "DNS Assist",
  connect: "Connect",
  protect: "Protect",
  network: "Network",
};

export function isPageId(value: string | null): value is PageId {
  return (
    value === "overview" ||
    value === "dns" ||
    value === "connect" ||
    value === "protect" ||
    value === "network"
  );
}

export function formatTag(tag: string): string {
  return tag.replace(/_/g, " ");
}

export function formatReason(reason: string): string {
  return reason.replace(/_/g, " ");
}
