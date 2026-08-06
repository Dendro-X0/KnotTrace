use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

const GITHUB_REPO: &str = "Dendro-X0/KnotTrace";
const USER_AGENT: &str = "KnotTrace-Updater";
const RELEASES_LATEST_URL: &str = "https://github.com/Dendro-X0/KnotTrace/releases/latest";

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

/// Field names must stay snake_case — the React UI reads `latest_version`, etc.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheck {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_url: Option<String>,
    pub notes: Option<String>,
    pub can_install_in_app: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgressEvent {
    pub phase: String,
    pub downloaded: u64,
    pub total: Option<u64>,
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> AppInfo {
    let info = app.package_info();
    AppInfo {
        name: info.name.to_string(),
        version: info.version.to_string(),
    }
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateCheck, String> {
    let current = app.package_info().version.to_string();

    if let Ok(updater) = app.updater() {
        match updater.check().await {
            Ok(Some(update)) => {
                return Ok(UpdateCheck {
                    available: true,
                    current_version: current,
                    latest_version: Some(update.version.clone()),
                    release_url: Some(release_tag_url(&update.version)),
                    notes: update.body.clone(),
                    can_install_in_app: true,
                });
            }
            Ok(None) => {
                return Ok(UpdateCheck {
                    available: false,
                    current_version: current,
                    latest_version: None,
                    release_url: None,
                    notes: None,
                    can_install_in_app: true,
                });
            }
            Err(error) => {
                tracing::warn!(target = "knottrace::updater", "signed updater check failed: {error}");
            }
        }
    }

    fallback_github_check(&current).await
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|error| error.to_string())?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|error| format!("Update check failed: {error}"))?
    else {
        return Err("No update is available.".to_string());
    };

    let handle = app.clone();
    let _ = handle.emit(
        "update-progress",
        UpdateProgressEvent {
            phase: "started".to_string(),
            downloaded: 0,
            total: None,
        },
    );

    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let progress_for_cb = std::sync::Arc::clone(&progress);

    update
        .download_and_install(
            {
                let handle = handle.clone();
                let progress_for_cb = std::sync::Arc::clone(&progress_for_cb);
                move |chunk_length, content_length| {
                    let downloaded =
                        progress_for_cb.fetch_add(chunk_length as u64, std::sync::atomic::Ordering::Relaxed)
                            + chunk_length as u64;
                    let _ = handle.emit(
                        "update-progress",
                        UpdateProgressEvent {
                            phase: "progress".to_string(),
                            downloaded,
                            total: content_length,
                        },
                    );
                }
            },
            {
                let handle = handle.clone();
                let progress_for_cb = progress_for_cb;
                move || {
                    let downloaded = progress_for_cb.load(std::sync::atomic::Ordering::Relaxed);
                    let _ = handle.emit(
                        "update-progress",
                        UpdateProgressEvent {
                            phase: "finished".to_string(),
                            downloaded,
                            total: None,
                        },
                    );
                }
            },
        )
        .await
        .map_err(|error| format!("Install failed: {error}"))?;

    Ok(())
}

async fn fallback_github_check(current: &str) -> Result<UpdateCheck, String> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let mut builder = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(12));

    if let Ok(environment) = network_core::detect_environment() {
        if environment.proxy.enabled {
            if let Some(proxy_server) = environment.proxy.server.as_deref() {
                let proxy_url = if proxy_server.contains("://") {
                    proxy_server.to_string()
                } else {
                    format!("http://{proxy_server}")
                };
                match reqwest::Proxy::all(&proxy_url) {
                    Ok(proxy) => {
                        builder = builder.proxy(proxy);
                    }
                    Err(error) => {
                        tracing::warn!(
                            target = "knottrace::updater",
                            "ignoring invalid system proxy for update check: {error}"
                        );
                    }
                }
            }
        }
    }

    let client = builder.build().map_err(|error| error.to_string())?;

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| {
            format!(
                "Could not reach GitHub for updates (proxy/VPN may be blocking api.github.com): {error}"
            )
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub returned {} while checking for updates.",
            response.status()
        ));
    }

    let release: GithubRelease = response
        .json()
        .await
        .map_err(|error| format!("Could not parse release info: {error}"))?;

    if release.draft || release.prerelease {
        return Ok(UpdateCheck {
            available: false,
            current_version: current.to_string(),
            latest_version: None,
            release_url: None,
            notes: None,
            can_install_in_app: false,
        });
    }

    let latest = release.tag_name.trim_start_matches('v').to_string();
    let available = version_is_newer(&latest, current);

    Ok(UpdateCheck {
        available,
        current_version: current.to_string(),
        latest_version: Some(latest),
        release_url: Some(if release.html_url.is_empty() {
            RELEASES_LATEST_URL.to_string()
        } else {
            release.html_url
        }),
        notes: release.body,
        can_install_in_app: false,
    })
}

fn release_tag_url(version: &str) -> String {
    let trimmed = version.trim().trim_start_matches('v');
    if trimmed.is_empty() {
        RELEASES_LATEST_URL.to_string()
    } else {
        format!("https://github.com/{GITHUB_REPO}/releases/tag/v{trimmed}")
    }
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

fn parse_version(value: &str) -> Vec<u32> {
    value
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_newer_versions() {
        assert!(version_is_newer("1.0.1", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("0.9.9", "1.0.0"));
    }

    #[test]
    fn update_check_serializes_snake_case_for_ui() {
        let payload = UpdateCheck {
            available: true,
            current_version: "1.6.0".to_string(),
            latest_version: Some("1.7.0".to_string()),
            release_url: Some(RELEASES_LATEST_URL.to_string()),
            notes: None,
            can_install_in_app: true,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("\"latest_version\":\"1.7.0\""));
        assert!(json.contains("\"current_version\":\"1.6.0\""));
        assert!(json.contains("\"can_install_in_app\":true"));
        assert!(json.contains("\"release_url\""));
        assert!(!json.contains("latestVersion"));
        assert!(!json.contains("canInstallInApp"));
    }

    #[test]
    fn release_tag_url_normalizes_version() {
        assert_eq!(
            release_tag_url("1.7.0"),
            "https://github.com/Dendro-X0/KnotTrace/releases/tag/v1.7.0"
        );
        assert_eq!(
            release_tag_url("v1.7.0"),
            "https://github.com/Dendro-X0/KnotTrace/releases/tag/v1.7.0"
        );
    }
}
