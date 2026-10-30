use chrono::Utc;
use network_core::{
    build_cloud_system_prompt, build_diagnostic_bundle_with_dir, parse_cloud_action_ids,
    sanitize_cloud_markdown, AiExplainProvider, AiExplainSettings, CloudExplainResult,
    DiagnosticBundleOptions, AI_EXPLAIN_CONSENT_VERSION, CLOUD_USER_PROMPT,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::state::{data_dir, AppState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiExplainSettingsUpdate {
    pub cloud_enabled: bool,
    pub provider: AiExplainProvider,
    pub base_url: String,
    pub model: String,
    pub include_ssid: bool,
    pub consent_ack: bool,
    pub api_key: Option<String>,
}

fn settings_path(dir: &Path) -> PathBuf {
    dir.join("ai_explain_settings.json")
}

fn key_path(dir: &Path) -> PathBuf {
    dir.join("ai_explain_api_key.txt")
}

fn load_key(dir: &Path) -> Option<String> {
    let path = key_path(dir);
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn save_key(dir: &Path, key: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    std::fs::write(key_path(dir), key.trim()).map_err(|error| error.to_string())
}

fn clear_key(dir: &Path) -> Result<(), String> {
    let path = key_path(dir);
    if path.exists() {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn load_settings_file(dir: &Path) -> Result<AiExplainSettings, String> {
    let path = settings_path(dir);
    if !path.exists() {
        return Ok(AiExplainSettings::default());
    }
    let contents = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let mut settings: AiExplainSettings =
        serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    settings.api_key_present = load_key(dir).is_some();
    Ok(settings)
}

fn save_settings_file(dir: &Path, settings: &AiExplainSettings) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let mut stored = settings.clone();
    // Never persist a raw key inside settings JSON.
    stored.api_key_present = load_key(dir).is_some();
    std::fs::write(
        settings_path(dir),
        serde_json::to_string_pretty(&stored).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_ai_explain_settings() -> Result<AiExplainSettings, String> {
    load_settings_file(&data_dir())
}

#[tauri::command]
pub fn set_ai_explain_settings(update: AiExplainSettingsUpdate) -> Result<AiExplainSettings, String> {
    let dir = data_dir();
    let mut settings = load_settings_file(&dir)?;

    if update.cloud_enabled {
        if !update.consent_ack && settings.consent_version != AI_EXPLAIN_CONSENT_VERSION {
            return Err(
                "Ack the AI data sheet (consent) before enabling cloud explain.".to_string(),
            );
        }
        if update.provider == AiExplainProvider::None {
            return Err("Choose an OpenAI-compatible provider before enabling cloud.".to_string());
        }
        if update.base_url.trim().is_empty() || update.model.trim().is_empty() {
            return Err("Base URL and model are required for cloud explain.".to_string());
        }
        let key = update
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| load_key(&dir));
        if key.is_none() {
            return Err("API key required to enable cloud explain.".to_string());
        }
        if let Some(value) = update.api_key.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            save_key(&dir, value)?;
        }
        settings.consent_version = AI_EXPLAIN_CONSENT_VERSION.to_string();
        settings.last_consent_at = Some(Utc::now());
    }

    settings.cloud_enabled = update.cloud_enabled;
    settings.provider = update.provider;
    settings.base_url = update.base_url.trim().trim_end_matches('/').to_string();
    settings.model = update.model.trim().to_string();
    settings.include_ssid = update.include_ssid;
    settings.api_key_present = load_key(&dir).is_some();

    if !settings.cloud_enabled {
        // Keep key on disk unless cleared explicitly; just mark disabled.
    }

    save_settings_file(&dir, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn clear_ai_explain_key() -> Result<AiExplainSettings, String> {
    let dir = data_dir();
    clear_key(&dir)?;
    let mut settings = load_settings_file(&dir)?;
    settings.cloud_enabled = false;
    settings.api_key_present = false;
    save_settings_file(&dir, &settings)?;
    Ok(settings)
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Option<Vec<ChatChoice>>,
    error: Option<ChatErrorBody>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: Option<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatErrorBody {
    message: Option<String>,
}

#[tauri::command]
pub async fn explain_with_cloud(
    state: State<'_, AppState>,
) -> Result<CloudExplainResult, String> {
    let dir = data_dir();
    let settings = load_settings_file(&dir)?;
    if !settings.cloud_enabled {
        return Err("Cloud explain is off. Enable it in settings after reviewing the data sheet.".to_string());
    }
    if settings.provider != AiExplainProvider::OpenaiCompat {
        return Err("Only OpenAI-compatible providers are supported in v1.".to_string());
    }
    if settings.consent_version != AI_EXPLAIN_CONSENT_VERSION {
        return Err("Consent version outdated — re-ack the data sheet.".to_string());
    }
    let api_key = load_key(&dir).ok_or_else(|| "API key missing. Save a key first.".to_string())?;

    let report = state
        .last_report
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?
        .clone()
        .ok_or_else(|| "Run a health check before cloud explain.".to_string())?;

    let bundle = build_diagnostic_bundle_with_dir(
        &report,
        DiagnosticBundleOptions {
            include_ssid: settings.include_ssid,
        },
        Some(&data_dir()),
    );
    let system = build_cloud_system_prompt(&bundle);
    let url = format!("{}/chat/completions", settings.base_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|error| error.to_string())?;

    let body = serde_json::json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": format!("{CLOUD_USER_PROMPT}\n\n{}", serde_json::to_string_pretty(&bundle).unwrap_or_default()) }
        ]
    });

    let response = client
        .post(&url)
        .bearer_auth(&api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("Cloud request failed: {error}"))?;

    let status = response.status();
    let payload: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|error| format!("Invalid provider response: {error}"))?;

    if !status.is_success() {
        let message = payload
            .error
            .and_then(|error| error.message)
            .unwrap_or_else(|| format!("Provider returned HTTP {status}"));
        return Ok(CloudExplainResult {
            ok: false,
            provider: "openai_compat".to_string(),
            model: settings.model,
            plain_markdown: String::new(),
            parsed_actions: Vec::new(),
            rejected_actions: Vec::new(),
            error: Some(message),
            received_at: Utc::now(),
        });
    }

    let raw = payload
        .choices
        .unwrap_or_default()
        .into_iter()
        .find_map(|choice| choice.message.and_then(|message| message.content))
        .unwrap_or_default();
    let plain = sanitize_cloud_markdown(&raw);
    let (parsed_actions, rejected_actions) = parse_cloud_action_ids(&plain);

    Ok(CloudExplainResult {
        ok: true,
        provider: "openai_compat".to_string(),
        model: settings.model,
        plain_markdown: plain,
        parsed_actions,
        rejected_actions,
        error: None,
        received_at: Utc::now(),
    })
}
