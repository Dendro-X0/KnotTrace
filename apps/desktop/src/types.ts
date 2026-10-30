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
  link_facts?: LinkFactsReport | null;
  local_caps?: LocalCapsReport | null;
  mtu_assist?: MtuAssistReport | null;
  local_repair?: LocalRepairReport | null;
  tunnel_compare?: TunnelPathCompareReport | null;
  upstream_pool?: UpstreamPoolProof | null;
  path_break?: PathBreakReport | null;
  integrity?: IntegrityBoardReport | null;
  egress_timeline?: EgressTimelineReport | null;
}

export type EgressTimelineClaim =
  | "none"
  | "insufficient_history"
  | "stable"
  | "ip_changed"
  | "churning"
  | "unstable";

export type EgressTimelineConfidence = "high" | "medium" | "low";

export type EgressTimelineEventKind = "ip_changed" | "unstable_sample";

export interface EgressTimelineEvent {
  at: string;
  kind: EgressTimelineEventKind;
  detail: string;
}

export interface EgressTimelineReport {
  claim: EgressTimelineClaim;
  confidence: EgressTimelineConfidence;
  title: string;
  summary: string;
  current_ip?: string | null;
  previous_ip?: string | null;
  distinct_ips: number;
  checks_considered: number;
  events: EgressTimelineEvent[];
  evidence: string[];
  not_proven: string[];
  action: string;
}

export interface AppCapabilities {
  companion_shell: boolean;
  background_monitor_default: boolean;
  platform: string;
}

export type IntegrityBoardLevel = "none" | "info" | "caution" | "elevated";
export type IntegrityBoardConfidence = "high" | "medium" | "low";
export type IntegrityIndicatorKind =
  | "hosts_file_delta"
  | "proxy_config_delta"
  | "proxy_enabled"
  | "dns_integrity"
  | "captive_portal"
  | "tls_intercept_hint"
  | "high_risk_listener"
  | "path_forced_routing"
  | "gateway_arp_churn"
  | "cert_store_delta"
  | "process_egress_hint";

export interface IntegrityIndicator {
  kind: IntegrityIndicatorKind;
  severity: "info" | "warning" | "critical";
  title: string;
  message: string;
  evidence: string;
}

export interface HighRiskListener {
  port: number;
  address: string;
  process_hint?: string | null;
}

export interface ListenerScanSummary {
  scanned: boolean;
  source: string;
  high_risk: HighRiskListener[];
  total_listened_sampled: number;
  summary: string;
}

export interface GatewayArpSnapshot {
  gateway_ip: string;
  mac?: string | null;
  source: string;
  observed: boolean;
}

export interface CertStoreSnapshot {
  root_count: number;
  fingerprint: string;
  source: string;
  observed: boolean;
}

export interface ProcessEgressHit {
  process_name: string;
  pid?: number | null;
  remote: string;
  count: number;
}

export interface ProcessEgressSnapshot {
  scanned: boolean;
  enabled: boolean;
  source: string;
  matched_targets: string[];
  hits: ProcessEgressHit[];
  summary: string;
}

export interface ProcessEgressSettings {
  enabled: boolean;
  consent_ack: boolean;
}

export interface IntegrityBoardReport {
  indicator_count: number;
  level: IntegrityBoardLevel;
  title: string;
  summary: string;
  disclaimer: string;
  indicators: IntegrityIndicator[];
  listeners?: ListenerScanSummary | null;
  gateway_arp?: GatewayArpSnapshot | null;
  cert_store?: CertStoreSnapshot | null;
  process_egress?: ProcessEgressSnapshot | null;
  not_checked: string[];
  confidence: IntegrityBoardConfidence;
}

export type PathBreakPath =
  | "direct"
  | "system_proxy"
  | "vpn_tunnel"
  | "tor_socks";

export type PathBreakHop =
  | "none"
  | "local"
  | "dns"
  | "tcp"
  | "tls"
  | "http"
  | "unknown";

export type PathEnvironmentClass =
  | "home_lan"
  | "guest_wifi"
  | "public_hotspot"
  | "captive_portal"
  | "corporate_proxy"
  | "vpn_only"
  | "tor"
  | "filtered_region"
  | "unknown";

export type PathBreakConfidence = "high" | "medium" | "low";

export interface PathBreakReport {
  local_ok: boolean;
  local_issues: string[];
  active_path: PathBreakPath;
  paths_present: PathBreakPath[];
  break_hop: PathBreakHop;
  environment_class: PathEnvironmentClass;
  confidence: PathBreakConfidence;
  title: string;
  summary: string;
  evidence: string[];
  not_checked: string[];
}

export interface LocalExplainStep {
  action_id: string;
  label: string;
  reason: string;
}

export interface LocalExplainReport {
  schema_version: string;
  headline: string;
  plain_summary: string;
  likely_cause?: string | null;
  next_steps: LocalExplainStep[];
  caveats: string[];
  source: string;
}

export type AiExplainProvider = "none" | "openai_compat";

export interface AiExplainSettings {
  cloud_enabled: boolean;
  provider: AiExplainProvider;
  base_url: string;
  model: string;
  api_key_present: boolean;
  include_ssid: boolean;
  consent_version: string;
  last_consent_at?: string | null;
}

export interface CloudExplainResult {
  ok: boolean;
  provider: string;
  model: string;
  plain_markdown: string;
  parsed_actions: string[];
  rejected_actions: string[];
  error?: string | null;
  received_at: string;
}

export interface DiagnosticBundle {
  schema_version: string;
  exported_at: string;
  app: { name: string; version?: string | null };
  redaction: { include_ssid: boolean; notes: string[] };
  os: string;
  environment_class?: PathEnvironmentClass | null;
  path_break?: {
    local_ok: boolean;
    active_path: PathBreakPath;
    break_hop: PathBreakHop;
    environment_class: PathEnvironmentClass;
    confidence: PathBreakConfidence;
    title: string;
    summary: string;
  } | null;
  score: { grade: HealthGrade; score: number; summary: string };
  diagnosis?: {
    slowdown_shape: string;
    confidence: string;
    primary_bottleneck?: string | null;
    summary: string;
  } | null;
  network_context?: {
    kind: string;
    risk_level: string;
    captive_state: string;
    summary: string;
  } | null;
  dns_integrity?: {
    state: string;
    confidence: string;
    mismatch_count: number;
    summary: string;
  } | null;
  proxy: { enabled: boolean; server?: string | null };
  tor: { detected: boolean; socks_reachable: boolean };
  vpn_detected: boolean;
  probe_table: { path: string; stage: string; ok: boolean; detail: string }[];
  link_summary?: string | null;
  upstream_pool?: {
    claim: UpstreamPoolClaim;
    confidence: UpstreamPoolConfidence;
    title: string;
  } | null;
  recommendations: {
    title: string;
    category: string;
    priority: number;
    action_id: string;
  }[];
  integrity_flags: string[];
  config_timeline: string[];
  not_checked: string[];
  ai_prompt_template: string;
}

export type UpstreamPoolClaim =
  | "none"
  | "inconclusive"
  | "active_path_impaired"
  | "active_path_recurring"
  | "upstream_pool_poor";

export type UpstreamPoolConfidence = "high" | "medium" | "low";

export interface UpstreamPoolProof {
  claim: UpstreamPoolClaim;
  confidence: UpstreamPoolConfidence;
  title: string;
  summary: string;
  evidence: string[];
  not_proven: string[];
  action: string;
  intermittent_domains: string[];
  recurring_impaired_checks: number;
  distinct_egress_ips: number;
  proxy_only_failure_domains: string[];
  checks_considered: number;
}

export type TunnelPathKind = "direct" | "system_proxy" | "tor_socks";

export interface TunnelPathSample {
  kind: TunnelPathKind;
  label: string;
  available: boolean;
  egress_ip: string | null;
  reachability: SiteReachResult[];
  median_latency_ms: number | null;
  success_count: number;
  failure_count: number;
  note: string | null;
}

export interface TunnelPathCompareReport {
  paths: TunnelPathSample[];
  tor_only_failures: string[];
  expectation: string;
  summary: string;
  vpn_detected: boolean;
  tor_detected: boolean;
  tor_socks_reachable: boolean;
  proxy_enabled: boolean;
}

export type LocalCapsIssueKind =
  | "autotuning_disabled"
  | "autotuning_restricted"
  | "adapter_power_saving";

export interface LocalCapsIssue {
  kind: LocalCapsIssueKind;
  severity: "info" | "warning" | "critical";
  title: string;
  message: string;
}

export interface LocalCapsReport {
  available: boolean;
  platform_note: string;
  tcp_autotuning_level: string | null;
  tcp_autotuning_ok: boolean;
  adapter_name: string | null;
  adapter_power_saving: boolean | null;
  issues: LocalCapsIssue[];
  summary: string;
  can_repair: boolean;
  repair_active: boolean;
}

export interface LocalCapsBackup {
  previous_autotuning_level: string;
  adapter_name: string | null;
  previous_allow_computer_turn_off: boolean | null;
  applied_at: string;
}

export interface LocalCapsState {
  available: boolean;
  can_repair: boolean;
  repair_active: boolean;
  backup: LocalCapsBackup | null;
  platform_note: string;
}

export interface LocalCapsRepairResult {
  kept: boolean;
  message: string;
}

export interface MtuAssistReport {
  available: boolean;
  platform_note: string;
  fragmentation_risk: boolean;
  tunnel_evidenced: boolean;
  interface_name: string | null;
  current_mtu: number | null;
  recommended_mtu: number | null;
  estimated_path_mtu: number | null;
  summary: string;
  can_repair: boolean;
  repair_active: boolean;
}

export interface MtuAssistBackup {
  interface_name: string;
  previous_mtu: number;
  applied_mtu: number;
  applied_at: string;
}

export interface MtuAssistState {
  available: boolean;
  can_repair: boolean;
  repair_active: boolean;
  backup: MtuAssistBackup | null;
  platform_note: string;
}

export interface MtuAssistRepairResult {
  kept: boolean;
  message: string;
}

export type LocalRepairActionKind = "flush_dns" | "renew_dhcp" | "restart_adapter";

export interface LocalRepairAction {
  kind: LocalRepairActionKind;
  title: string;
  message: string;
  recommended: boolean;
  available: boolean;
}

export interface LocalRepairReport {
  available: boolean;
  platform_note: string;
  actions: LocalRepairAction[];
  summary: string;
  local_problem_likely: boolean;
}

export interface LocalRepairResult {
  kind: LocalRepairActionKind;
  success: boolean;
  message: string;
}

export type LinkDuplex = "full" | "half" | "unknown";
export type LinkIssueKind =
  | "ethernet_capped"
  | "half_duplex"
  | "prefer_ethernet"
  | "wifi_active"
  | "wifi_weak_signal";
export type WifiBand = "band_2_4" | "band_5" | "band_6" | "unknown";

export interface LinkAdapterFact {
  name: string;
  friendly_name: string | null;
  kind: string;
  is_up: boolean;
  is_default_route: boolean;
  speed_mbps: number | null;
  duplex: LinkDuplex | null;
  media: string | null;
  raw_speed: string | null;
  wifi_ssid?: string | null;
  wifi_signal_percent?: number | null;
  wifi_signal_dbm?: number | null;
  wifi_channel?: number | null;
  wifi_band?: WifiBand | null;
  wifi_radio?: string | null;
}

export interface LinkIssue {
  kind: LinkIssueKind;
  severity: "info" | "warning" | "critical";
  title: string;
  message: string;
}

export interface LinkFactsReport {
  active: LinkAdapterFact | null;
  adapters: LinkAdapterFact[];
  issues: LinkIssue[];
  summary: string;
  source: string;
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
  | "link_local"
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
  do_not_disturb: boolean;
  notify_digest_only: boolean;
  quiet_hours_enabled: boolean;
  quiet_hours_start: string;
  quiet_hours_end: string;
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

export interface AutoProtectLogEntry {
  timestamp: string;
  kind: string;
  success: boolean;
  message: string;
  trigger: string;
  check_reason: string;
  rollback_hint: string;
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
  proxy_enabled?: boolean | null;
  proxy_only_failure_count?: number | null;
  likely_provider_side?: boolean | null;
  egress_ip?: string | null;
  upstream_claim?: UpstreamPoolClaim | null;
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
