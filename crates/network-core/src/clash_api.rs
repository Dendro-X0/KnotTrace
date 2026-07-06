use crate::types::ConnectConfig;
use serde::Deserialize;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

pub const DEFAULT_API_BASES: &[&str] = &[
    "http://127.0.0.1:9090",
    "http://127.0.0.1:9091",
    "http://127.0.0.1:6170",
];

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProxiesResponse {
    proxies: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DelayResponse {
    delay: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum ClashApiError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("api returned {status}: {body}")]
    Api { status: u16, body: String },
}

pub struct ClashClient {
    http: reqwest::Client,
    config: ConnectConfig,
}

impl ClashClient {
    pub fn new(config: ConnectConfig) -> Result<Self, ClashApiError> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| ClashApiError::Request(error.to_string()))?;

        Ok(Self { http, config })
    }

    pub async fn version(&self) -> Result<String, ClashApiError> {
        let response: VersionResponse = self.get("/version").await?;
        Ok(response.version.unwrap_or_else(|| "unknown".to_string()))
    }

    pub async fn proxies(&self) -> Result<serde_json::Map<String, serde_json::Value>, ClashApiError> {
        let response: ProxiesResponse = self.get("/proxies").await?;
        Ok(response.proxies)
    }

    pub async fn proxy_delay(
        &self,
        proxy_name: &str,
        test_url: &str,
        timeout_ms: u32,
    ) -> Result<Option<u32>, ClashApiError> {
        let path = format!(
            "/proxies/{}/delay?timeout={timeout_ms}&url={}",
            urlencoding_encode(proxy_name),
            urlencoding_encode(test_url)
        );
        let response: DelayResponse = self.get(&path).await?;
        Ok(response.delay)
    }

    pub async fn select_proxy(&self, group: &str, target: &str) -> Result<(), ClashApiError> {
        let url = format!(
            "{}/proxies/{}",
            self.config.api_base.trim_end_matches('/'),
            urlencoding_encode(group)
        );
        let request = self
            .http
            .put(url)
            .json(&serde_json::json!({ "name": target }));

        let response = self
            .send(request)
            .await?
            .error_for_status()
            .map_err(|error| {
                let status = error.status().map(|value| value.as_u16()).unwrap_or(0);
                ClashApiError::Api {
                    status,
                    body: error.to_string(),
                }
            })?;

        let _ = response.text().await;
        Ok(())
    }

    pub async fn probe_endpoint(api_base: &str, secret: Option<&str>) -> Result<String, ClashApiError> {
        let client = ClashClient::new(ConnectConfig {
            api_base: api_base.to_string(),
            secret: secret.map(str::to_string),
            auto_discovered: true,
        })?;
        client.version().await
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ClashApiError> {
        let url = format!(
            "{}{}",
            self.config.api_base.trim_end_matches('/'),
            path
        );
        let response = self.send(self.http.get(url)).await?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ClashApiError::Request(error.to_string()))?;

        if !status.is_success() {
            return Err(ClashApiError::Api {
                status: status.as_u16(),
                body,
            });
        }

        serde_json::from_str(&body).map_err(|error| ClashApiError::Request(error.to_string()))
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response, ClashApiError> {
        let request = if let Some(secret) = &self.config.secret {
            request.header("Authorization", format!("Bearer {secret}"))
        } else {
            request
        };

        request
            .send()
            .await
            .map_err(|error| ClashApiError::Request(error.to_string()))
    }
}

fn urlencoding_encode(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
            _ => format!("%{:02X}", ch as u8),
        })
        .collect()
}
