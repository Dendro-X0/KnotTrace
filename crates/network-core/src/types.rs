use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    Ethernet,
    WiFi,
    Cellular,
    Loopback,
    Virtual,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentTag {
    HomeLan,
    Vpn,
    Proxy,
    Tor,
    Public,
    Datacenter,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServer {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub friendly_name: Option<String>,
    pub kind: LinkKind,
    pub is_up: bool,
    pub ipv4: Vec<String>,
    pub gateway: Option<String>,
    pub is_default_route: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub enabled: bool,
    pub server: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorStatus {
    pub detected: bool,
    pub socks_endpoint: Option<String>,
    pub socks_reachable: bool,
    pub source: String,
}

pub fn default_tor_status() -> TorStatus {
    TorStatus {
        detected: false,
        socks_endpoint: None,
        socks_reachable: false,
        source: "none".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub hostname: String,
    pub interfaces: Vec<NetworkInterface>,
    pub dns_servers: Vec<DnsServer>,
    pub proxy: ProxySettings,
    pub tags: Vec<EnvironmentTag>,
    pub default_gateway: Option<String>,
    pub active_interface: Option<String>,
    #[serde(default = "default_tor_status")]
    pub tor: TorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySample {
    pub target: String,
    pub avg_ms: f64,
    pub loss_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsProbe {
    pub resolver: String,
    pub query: String,
    pub latency_ms: f64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub gateway: Option<LatencySample>,
    pub internet: Option<LatencySample>,
    pub dns: Vec<DnsProbe>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BufferbloatGrade {
    None,
    Mild,
    Moderate,
    Severe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferbloatProbe {
    pub idle_latency_ms: f64,
    pub loaded_latency_ms: f64,
    pub latency_delta_ms: f64,
    pub grade: BufferbloatGrade,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtuProbe {
    pub estimated_path_mtu: Option<u16>,
    pub recommended_tcp_mss: Option<u16>,
    pub fragmentation_risk: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityProbeResult {
    pub bufferbloat: Option<BufferbloatProbe>,
    pub mtu: Option<MtuProbe>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthGrade {
    Good,
    Fair,
    Poor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub grade: HealthGrade,
    pub score: u8,
    pub summary: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub environment: EnvironmentSnapshot,
    pub probe: ProbeResult,
    pub score: HealthScore,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_integrity: Option<DnsIntegrityStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnosis: Option<NetworkDiagnosis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stability: Option<StabilityProbeResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_reachability: Option<SiteReachabilityStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub egress: Option<EgressReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_context: Option<NetworkContextReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendations: Option<NetworkRecommendations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsCandidateResult {
    pub resolver: String,
    pub label: String,
    pub latency_ms: f64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsAssistRecommendation {
    pub current_latency_ms: Option<f64>,
    pub current_resolvers: Vec<String>,
    pub candidates: Vec<DnsCandidateResult>,
    pub recommended: Option<DnsCandidateResult>,
    pub should_apply: bool,
    pub reason: String,
    pub interface_alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsBackup {
    pub interface_alias: String,
    pub previous_servers: Vec<String>,
    pub use_dhcp: bool,
    pub applied_resolver: String,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsAssistState {
    pub active: bool,
    pub backup: Option<DnsBackup>,
    pub can_apply: bool,
    pub platform_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsApplyResult {
    pub applied_resolver: String,
    pub before_latency_ms: f64,
    pub after_latency_ms: f64,
    pub kept: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyKernelKind {
    Mihomo,
    SingBox,
    Clash,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectConfig {
    pub api_base: String,
    pub secret: Option<String>,
    pub auto_discovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyKernelInfo {
    pub kind: ProxyKernelKind,
    pub api_base: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMemberView {
    pub name: String,
    pub proxy_type: String,
    pub delay_ms: Option<u32>,
    pub alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroupView {
    pub name: String,
    pub group_type: String,
    pub current: Option<String>,
    pub members: Vec<ProxyMemberView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySwitchRecommendation {
    pub group_name: String,
    pub current_proxy: Option<String>,
    pub target_proxy: String,
    pub current_delay_ms: Option<u32>,
    pub target_delay_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRecommendation {
    pub kernel: Option<ProxyKernelInfo>,
    pub groups: Vec<ProxyGroupView>,
    pub recommended_switch: Option<ProxySwitchRecommendation>,
    pub should_apply: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectApplyResult {
    pub group_name: String,
    pub target_proxy: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    Trusted,
    Caution,
    Untrusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectActionKind {
    DnsAssist,
    ConnectAssist,
    RunCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectAction {
    pub kind: ProtectActionKind,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectAlert {
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub actions: Vec<ProtectAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectSettings {
    pub enabled: bool,
    pub notify_on_grade_drop: bool,
    pub notify_on_untrusted_network: bool,
    pub notify_on_degraded: bool,
    #[serde(default = "default_auto_apply_dns")]
    pub auto_apply_dns: bool,
    #[serde(default = "default_auto_apply_connect")]
    pub auto_apply_connect: bool,
    #[serde(default = "default_auto_apply_untrusted_only")]
    pub auto_apply_on_untrusted_only: bool,
    /// Auto-apply DNS when integrity checks detect poisoning or hijacking.
    #[serde(default = "default_auto_recover_dns_integrity")]
    pub auto_recover_dns_integrity: bool,
    /// Auto-switch proxy nodes when verification sites fail over HTTPS.
    #[serde(default = "default_auto_recover_site_access")]
    pub auto_recover_site_access: bool,
}

fn default_auto_apply_dns() -> bool {
    true
}

fn default_auto_apply_connect() -> bool {
    false
}

fn default_auto_recover_dns_integrity() -> bool {
    true
}

fn default_auto_recover_site_access() -> bool {
    false
}

fn default_auto_apply_untrusted_only() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectStatus {
    pub trust_level: TrustLevel,
    pub alerts: Vec<ProtectAlert>,
    pub settings: ProtectSettings,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub score: u8,
    pub grade: HealthGrade,
    pub dns_latency_ms: Option<f64>,
    pub internet_latency_ms: Option<f64>,
    pub gateway_latency_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_integrity_state: Option<DnsIntegrityState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_integrity_confidence: Option<DnsIntegrityConfidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_integrity_mismatch_count: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoProtectAction {
    pub kind: String,
    pub message: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoProtectResult {
    pub applied: Vec<AutoProtectAction>,
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DnsIntegrityState {
    Ok,
    Caution,
    Suspicious,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DnsIntegrityConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsIntegrityFinding {
    pub domain: String,
    pub local_answers: Vec<String>,
    pub trusted_answers: Vec<String>,
    pub local_error: Option<String>,
    pub trusted_error_count: u8,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsIntegrityStatus {
    pub state: DnsIntegrityState,
    pub confidence: DnsIntegrityConfidence,
    pub mismatch_count: u8,
    pub checked_domains: u8,
    pub summary: String,
    pub details: Vec<DnsIntegrityFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DnsIntegritySettings {
    pub verification_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteReachResult {
    pub domain: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteReachabilityStatus {
    pub checked_domains: u8,
    pub success_count: u8,
    pub failure_count: u8,
    pub results: Vec<SiteReachResult>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BottleneckCategory {
    InternetUnreachable,
    InternetLoss,
    InternetLatency,
    Gateway,
    DnsFailure,
    DnsSlow,
    DnsIntegrity,
    SiteAccess,
    ProxyPath,
    VpnTunnel,
    TorTunnel,
    Bufferbloat,
    MtuFragmentation,
    WifiPath,
    CellularPath,
    PublicNetwork,
    CaptivePortal,
    EgressUnstable,
    Healthy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EgressConfidence {
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EgressPathKind {
    System,
    TorSocks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressEndpointResult {
    pub provider: String,
    pub ip: Option<String>,
    pub latency_ms: Option<f64>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressPathReport {
    pub kind: EgressPathKind,
    pub primary_ip: Option<String>,
    pub endpoints: Vec<EgressEndpointResult>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressReport {
    pub primary_ip: Option<String>,
    pub confidence: EgressConfidence,
    pub system_path: EgressPathReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tor_path: Option<EgressPathReport>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkContextKind {
    HomeLan,
    GuestWifi,
    PublicCellular,
    CaptivePortal,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkRiskLevel {
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptivePortalState {
    NotDetected,
    Suspected,
    Confirmed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptivePortalStatus {
    pub state: CaptivePortalState,
    pub probe_url: String,
    pub status_code: Option<u16>,
    pub redirected: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContextReport {
    pub kind: NetworkContextKind,
    pub risk_level: NetworkRiskLevel,
    pub captive_portal: CaptivePortalStatus,
    pub signals: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    PublicNetwork,
    CaptivePortal,
    DnsSecurity,
    VpnPrivacy,
    ProxyPath,
    TorPath,
    Egress,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRecommendation {
    pub category: RecommendationCategory,
    pub priority: u8,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRecommendations {
    pub items: Vec<NetworkRecommendation>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckHint {
    pub category: BottleneckCategory,
    pub severity: AlertLevel,
    pub title: String,
    pub message: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnosis {
    pub summary: String,
    pub primary_bottleneck: Option<BottleneckCategory>,
    pub hints: Vec<BottleneckHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSpeedtestNote {
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub ping_ms: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkProbeSummary {
    pub internet_latency_ms: Option<f64>,
    pub internet_loss_pct: Option<f64>,
    pub gateway_latency_ms: Option<f64>,
    pub dns_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSnapshot {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub label: String,
    pub environment_fingerprint: String,
    pub health_score: u8,
    pub health_grade: HealthGrade,
    pub probe_summary: BenchmarkProbeSummary,
    pub dns_integrity_state: Option<DnsIntegrityState>,
    pub primary_bottleneck: Option<BottleneckCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_speedtest: Option<ExternalSpeedtestNote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThroughputSettings {
    pub download_bytes: u64,
    pub upload_bytes: u64,
}

pub fn default_throughput_settings() -> ThroughputSettings {
    ThroughputSettings {
        download_bytes: 5_000_000,
        upload_bytes: 2_000_000,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputProbeResult {
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub download_bytes: u64,
    pub upload_bytes: u64,
    pub duration_ms: u64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPathSample {
    pub proxy_name: String,
    pub delay_ms: Option<u32>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPathComparison {
    pub group_name: String,
    pub test_url: String,
    pub samples: Vec<ProxyPathSample>,
    pub fastest_proxy: Option<String>,
    pub summary: String,
}
