use crate::clash_api::{ClashApiError, ClashClient, DEFAULT_API_BASES};
use crate::types::*;
use std::path::Path;
use thiserror::Error;

const GROUP_TYPES: &[&str] = &["Selector", "URLTest", "Fallback", "LoadBalance"];
const DELAY_IMPROVEMENT_MIN_MS: u32 = 50;
const DELAY_IMPROVEMENT_MIN_RATIO: f64 = 0.25;

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("clash api error: {0}")]
    Api(#[from] ClashApiError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("connect unavailable: {0}")]
    Unavailable(String),
}

pub fn config_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("connect_config.json")
}

pub fn load_connect_config(data_dir: &Path) -> Result<Option<ConnectConfig>, ConnectError> {
    let path = config_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&contents)?))
}

pub fn save_connect_config(data_dir: &Path, config: &ConnectConfig) -> Result<(), ConnectError> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(config_path(data_dir), serde_json::to_string_pretty(config)?)?;
    Ok(())
}

pub async fn discover_connect_config(data_dir: &Path) -> Result<Option<ConnectConfig>, ConnectError> {
    if let Some(existing) = load_connect_config(data_dir)? {
        if ClashClient::new(existing.clone())?
            .version()
            .await
            .is_ok()
        {
            return Ok(Some(existing));
        }
    }

    for api_base in DEFAULT_API_BASES {
        if ClashClient::probe_endpoint(api_base, None).await.is_ok() {
            let config = ConnectConfig {
                api_base: (*api_base).to_string(),
                secret: None,
                auto_discovered: true,
            };
            save_connect_config(data_dir, &config)?;
            return Ok(Some(config));
        }
    }

    Ok(None)
}

pub async fn recommend_connect(
    config: &ConnectConfig,
    environment: &EnvironmentSnapshot,
    health: Option<&HealthScore>,
) -> Result<ConnectRecommendation, ConnectError> {
    let client = ClashClient::new(config.clone())?;
    let version = client.version().await.ok();
    let kernel = ProxyKernelInfo {
        kind: classify_kernel(version.as_deref()),
        api_base: config.api_base.clone(),
        version,
    };

    let proxies = client.proxies().await?;
    let groups = parse_proxy_groups(&proxies);

    let recommended_switch = pick_recommended_switch(&groups, environment, health);
    let (should_apply, reason) = evaluate_connect_recommendation(
        &kernel,
        environment,
        health,
        &recommended_switch,
    );

    Ok(ConnectRecommendation {
        kernel: Some(kernel),
        groups,
        recommended_switch,
        should_apply,
        reason,
    })
}

pub async fn recommend_connect_discovered(
    data_dir: &Path,
    environment: &EnvironmentSnapshot,
    health: Option<&HealthScore>,
) -> Result<ConnectRecommendation, ConnectError> {
    let Some(config) = discover_connect_config(data_dir).await? else {
        return Ok(ConnectRecommendation {
            kernel: None,
            groups: Vec::new(),
            recommended_switch: None,
            should_apply: false,
            reason:
                "No Mihomo or sing-box Clash API found on 127.0.0.1:9090/9091/6170. Enable external-controller in your proxy app."
                    .to_string(),
        });
    };

    recommend_connect(&config, environment, health).await
}

pub const DEFAULT_PROXY_TEST_URL: &str = "http://www.gstatic.com/generate_204";
const PROXY_DELAY_TIMEOUT_MS: u32 = 5000;
const MAX_PROXY_SAMPLES: usize = 12;

pub async fn compare_proxy_paths(
    config: &ConnectConfig,
    group_name: &str,
) -> Result<ProxyPathComparison, ConnectError> {
    let client = ClashClient::new(config.clone())?;
    let proxies = client.proxies().await?;
    let groups = parse_proxy_groups(&proxies);

    let group = groups
        .iter()
        .find(|group| group.name == group_name)
        .ok_or_else(|| {
            ConnectError::Unavailable(format!("Proxy group '{group_name}' was not found."))
        })?;

    let current = group.current.as_deref();
    let mut samples = Vec::new();

    for member in group.members.iter().take(MAX_PROXY_SAMPLES) {
        let delay_ms = client
            .proxy_delay(&member.name, DEFAULT_PROXY_TEST_URL, PROXY_DELAY_TIMEOUT_MS)
            .await
            .ok()
            .flatten();

        samples.push(ProxyPathSample {
            proxy_name: member.name.clone(),
            delay_ms,
            is_current: current == Some(member.name.as_str()),
        });
    }

    let fastest_proxy = samples
        .iter()
        .filter(|sample| sample.delay_ms.is_some())
        .min_by_key(|sample| sample.delay_ms.unwrap_or(u32::MAX))
        .map(|sample| sample.proxy_name.clone());

    let summary = build_proxy_comparison_summary(group_name, current, &fastest_proxy, &samples);

    Ok(ProxyPathComparison {
        group_name: group_name.to_string(),
        test_url: DEFAULT_PROXY_TEST_URL.to_string(),
        samples,
        fastest_proxy,
        summary,
    })
}

pub async fn compare_proxy_paths_discovered(
    data_dir: &Path,
    group_name: &str,
) -> Result<ProxyPathComparison, ConnectError> {
    let Some(config) = discover_connect_config(data_dir).await? else {
        return Err(ConnectError::Unavailable(
            "No Mihomo or sing-box Clash API found. Enable external-controller in your proxy app."
                .to_string(),
        ));
    };

    compare_proxy_paths(&config, group_name).await
}

fn build_proxy_comparison_summary(
    group_name: &str,
    current: Option<&str>,
    fastest_proxy: &Option<String>,
    samples: &[ProxyPathSample],
) -> String {
    let tested = samples
        .iter()
        .filter(|sample| sample.delay_ms.is_some())
        .count();

    if tested == 0 {
        return format!("No delay results for {group_name}. Run a latency test in your proxy app first.");
    }

    let Some(fastest) = fastest_proxy else {
        return format!("Tested {tested} nodes in {group_name}, but no fastest node was identified.");
    };

    if current == Some(fastest.as_str()) {
        return format!(
            "Current node {fastest} is the fastest of {tested} tested nodes in {group_name}."
        );
    }

    let current_note = current
        .map(|name| format!(" Current node is {name}."))
        .unwrap_or_default();

    format!(
        "Fastest tested node in {group_name} is {fastest} ({tested} nodes).{current_note}"
    )
}

pub async fn apply_connect_switch(
    config: &ConnectConfig,
    group_name: &str,
    target_proxy: &str,
) -> Result<ConnectApplyResult, ConnectError> {
    let client = ClashClient::new(config.clone())?;
    client.select_proxy(group_name, target_proxy).await?;

    Ok(ConnectApplyResult {
        group_name: group_name.to_string(),
        target_proxy: target_proxy.to_string(),
        message: format!("Switched {group_name} to {target_proxy}."),
    })
}

fn classify_kernel(version: Option<&str>) -> ProxyKernelKind {
    let Some(version) = version else {
        return ProxyKernelKind::Unknown;
    };

    let lower = version.to_lowercase();
    if lower.contains("mihomo") || lower.contains("meta") {
        ProxyKernelKind::Mihomo
    } else if lower.contains("sing") {
        ProxyKernelKind::SingBox
    } else if lower.contains("clash") {
        ProxyKernelKind::Clash
    } else {
        ProxyKernelKind::Unknown
    }
}

fn parse_proxy_groups(
    proxies: &serde_json::Map<String, serde_json::Value>,
) -> Vec<ProxyGroupView> {
    let mut groups = Vec::new();

    for (name, value) in proxies {
        let Some(obj) = value.as_object() else {
            continue;
        };
        let group_type = obj
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();

        if !GROUP_TYPES.iter().any(|kind| kind == &group_type) {
            continue;
        }

        let current = obj.get("now").and_then(|value| value.as_str()).map(str::to_string);
        let member_names = obj
            .get("all")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let members = member_names
            .into_iter()
            .filter_map(|member_name| {
                proxies.get(&member_name).map(|member| {
                    let member_obj = member.as_object();
                    let proxy_type = member_obj
                        .and_then(|value| value.get("type"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let delay_ms = extract_delay(member);
                    ProxyMemberView {
                        name: member_name,
                        proxy_type,
                        delay_ms,
                        alive: delay_ms.is_some(),
                    }
                })
            })
            .collect();

        groups.push(ProxyGroupView {
            name: name.clone(),
            group_type,
            current,
            members,
        });
    }

    groups.sort_by(|left, right| left.name.cmp(&right.name));
    groups
}

fn extract_delay(proxy: &serde_json::Value) -> Option<u32> {
    if let Some(delay) = proxy.get("delay").and_then(|value| value.as_u64()) {
        return Some(delay as u32);
    }

    proxy
        .get("history")
        .and_then(|value| value.as_array())
        .and_then(|history| history.last())
        .and_then(|entry| entry.get("delay"))
        .and_then(|value| value.as_u64())
        .map(|delay| delay as u32)
}

fn pick_recommended_switch(
    groups: &[ProxyGroupView],
    environment: &EnvironmentSnapshot,
    health: Option<&HealthScore>,
) -> Option<ProxySwitchRecommendation> {
    let needs_help = health.is_some_and(|score| {
        matches!(score.grade, HealthGrade::Fair | HealthGrade::Poor)
    }) || environment.tags.contains(&EnvironmentTag::Proxy);

    if !needs_help {
        return None;
    }

    let preferred_groups = groups
        .iter()
        .filter(|group| is_preferred_group(&group.name, &group.group_type))
        .collect::<Vec<_>>();

    let candidate_groups = if preferred_groups.is_empty() {
        groups.iter().collect::<Vec<_>>()
    } else {
        preferred_groups
    };

    let mut best: Option<ProxySwitchRecommendation> = None;

    for group in candidate_groups {
        let current_name = group.current.clone();
        let current_delay = current_name
            .as_ref()
            .and_then(|name| group.members.iter().find(|member| &member.name == name))
            .and_then(|member| member.delay_ms);

        let target = group
            .members
            .iter()
            .filter(|member| member.alive)
            .min_by_key(|member| member.delay_ms.unwrap_or(u32::MAX))?;

        if current_name.as_deref() == Some(target.name.as_str()) {
            continue;
        }

        let current_delay_ms = current_delay.unwrap_or(u32::MAX);
        let target_delay_ms = target.delay_ms.unwrap_or(u32::MAX);
        let improvement = current_delay_ms.saturating_sub(target_delay_ms);

        if current_delay_ms != u32::MAX
            && improvement < DELAY_IMPROVEMENT_MIN_MS
            && (improvement as f64) < current_delay_ms as f64 * DELAY_IMPROVEMENT_MIN_RATIO
        {
            continue;
        }

        let candidate = ProxySwitchRecommendation {
            group_name: group.name.clone(),
            current_proxy: current_name,
            target_proxy: target.name.clone(),
            current_delay_ms: current_delay,
            target_delay_ms: target.delay_ms,
        };

        if best
            .as_ref()
            .map(|value| {
                value.target_delay_ms.unwrap_or(u32::MAX)
                    > candidate.target_delay_ms.unwrap_or(u32::MAX)
            })
            .unwrap_or(true)
        {
            best = Some(candidate);
        }
    }

    best
}

fn is_preferred_group(name: &str, group_type: &str) -> bool {
    let lower = name.to_lowercase();
    group_type == "URLTest"
        || lower.contains("auto")
        || lower.contains("proxy")
        || lower.contains("select")
        || lower.contains("节点选择")
        || lower.contains("自动")
}

fn evaluate_connect_recommendation(
    kernel: &ProxyKernelInfo,
    environment: &EnvironmentSnapshot,
    health: Option<&HealthScore>,
    recommendation: &Option<ProxySwitchRecommendation>,
) -> (bool, String) {
    let kernel_name = format!("{:?}", kernel.kind).to_lowercase();

    let Some(switch) = recommendation else {
        if environment.proxy.enabled || environment.tags.contains(&EnvironmentTag::Proxy) {
            return (
                false,
                format!(
                    "{kernel_name} is connected, but no faster proxy node stood out. Try running a latency test in your proxy app first."
                ),
            );
        }

        return (
            false,
            format!(
                "{kernel_name} API detected. Connection health looks acceptable for the current path."
            ),
        );
    };

    let health_note = health
        .map(|score| format!(" Current health score is {}.", score.score))
        .unwrap_or_default();

    let current = switch
        .current_delay_ms
        .map(|delay| format!("{delay} ms"))
        .unwrap_or_else(|| "unknown latency".to_string());
    let target = switch
        .target_delay_ms
        .map(|delay| format!("{delay} ms"))
        .unwrap_or_else(|| "unknown latency".to_string());

    (
        true,
        format!(
            "{} could switch {} from {} ({}) to {} ({}).{health_note}",
            kernel_name,
            switch.group_name,
            switch.current_proxy.as_deref().unwrap_or("none"),
            current,
            switch.target_proxy,
            target
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_selector_groups_from_clash_payload() {
        let mut proxies = serde_json::Map::new();
        proxies.insert(
            "AUTO".to_string(),
            json!({
                "type": "URLTest",
                "now": "HK-1",
                "all": ["HK-1", "US-1"]
            }),
        );
        proxies.insert(
            "HK-1".to_string(),
            json!({ "type": "Shadowsocks", "history": [{"delay": 120}] }),
        );
        proxies.insert(
            "US-1".to_string(),
            json!({ "type": "Shadowsocks", "history": [{"delay": 40}] }),
        );

        let groups = parse_proxy_groups(&proxies);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].members.len(), 2);
    }

    #[test]
    fn summarizes_proxy_comparison() {
        let summary = build_proxy_comparison_summary(
            "AUTO",
            Some("HK-1"),
            &Some("US-1".to_string()),
            &[
                ProxyPathSample {
                    proxy_name: "HK-1".to_string(),
                    delay_ms: Some(120),
                    is_current: true,
                },
                ProxyPathSample {
                    proxy_name: "US-1".to_string(),
                    delay_ms: Some(40),
                    is_current: false,
                },
            ],
        );
        assert!(summary.contains("US-1"));
        assert!(summary.contains("HK-1"));
    }
}
