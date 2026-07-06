use crate::types::*;
use crate::CoreError;
use std::path::Path;
use thiserror::Error;

const TRUSTED_RESOLVERS: &[&str] = &["1.1.1.1", "8.8.8.8", "9.9.9.9"];
const LOCAL_RESOLVER: &str = "system";
const MAX_VERIFICATION_DOMAINS: usize = 12;

pub const DEFAULT_VERIFICATION_DOMAINS: &[&str] =
    &["example.com", "cloudflare.com", "microsoft.com", "github.com"];

#[derive(Debug, Error)]
pub enum DnsIntegrityError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid settings: {0}")]
    Invalid(String),
}

pub fn settings_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("dns_integrity_settings.json")
}

pub fn default_dns_integrity_settings() -> DnsIntegritySettings {
    DnsIntegritySettings {
        verification_domains: DEFAULT_VERIFICATION_DOMAINS
            .iter()
            .map(|domain| (*domain).to_string())
            .collect(),
    }
}

pub fn load_dns_integrity_settings(data_dir: &Path) -> Result<DnsIntegritySettings, DnsIntegrityError> {
    let path = settings_path(data_dir);
    if !path.exists() {
        return Ok(default_dns_integrity_settings());
    }

    let contents = std::fs::read_to_string(path)?;
    let settings: DnsIntegritySettings = serde_json::from_str(&contents)?;
    Ok(DnsIntegritySettings {
        verification_domains: normalize_verification_domains(&settings.verification_domains)?,
    })
}

pub fn save_dns_integrity_settings(
    data_dir: &Path,
    settings: &DnsIntegritySettings,
) -> Result<DnsIntegritySettings, DnsIntegrityError> {
    let normalized = DnsIntegritySettings {
        verification_domains: normalize_verification_domains(&settings.verification_domains)?,
    };
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        settings_path(data_dir),
        serde_json::to_string_pretty(&normalized)?,
    )?;
    Ok(normalized)
}

pub fn normalize_verification_domains(domains: &[String]) -> Result<Vec<String>, DnsIntegrityError> {
    let mut normalized = Vec::new();

    for domain in domains {
        let trimmed = domain.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if !is_valid_domain(&trimmed) {
            return Err(DnsIntegrityError::Invalid(format!("Invalid domain: {domain}")));
        }
        if !normalized.contains(&trimmed) {
            normalized.push(trimmed);
        }
    }

    if normalized.is_empty() {
        return Err(DnsIntegrityError::Invalid(
            "At least one verification domain is required".to_string(),
        ));
    }

    if normalized.len() > MAX_VERIFICATION_DOMAINS {
        return Err(DnsIntegrityError::Invalid(format!(
            "At most {MAX_VERIFICATION_DOMAINS} verification domains are allowed"
        )));
    }

    Ok(normalized)
}

fn is_valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }

    domain.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
    })
}

pub async fn evaluate_dns_integrity(
    _environment: &EnvironmentSnapshot,
    settings: &DnsIntegritySettings,
) -> Result<DnsIntegrityStatus, CoreError> {
    let mut findings = Vec::new();

    for domain in &settings.verification_domains {
        if let Some(finding) = check_domain_integrity(domain).await {
            findings.push(finding);
        }
    }

    Ok(classify_integrity_findings(
        findings,
        settings.verification_domains.len() as u8,
    ))
}

async fn check_domain_integrity(domain: &str) -> Option<DnsIntegrityFinding> {
    let local_result = crate::probe::resolve_dns_addresses(LOCAL_RESOLVER, domain).await;
    let mut trusted_answers = Vec::new();
    let mut trusted_error_count = 0u8;

    for resolver in TRUSTED_RESOLVERS {
        match crate::probe::resolve_dns_addresses(resolver, domain).await {
            Ok(mut answers) => trusted_answers.append(&mut answers),
            Err(_) => trusted_error_count = trusted_error_count.saturating_add(1),
        }
    }

    trusted_answers.sort();
    trusted_answers.dedup();

    match local_result {
        Ok(local_answers) => {
            if trusted_answers.is_empty() {
                return None;
            }
            if answers_overlap(&local_answers, &trusted_answers) {
                return None;
            }
            Some(DnsIntegrityFinding {
                domain: domain.to_string(),
                local_answers,
                trusted_answers,
                local_error: None,
                trusted_error_count,
                reason: "Local resolver returned different addresses than trusted public DNS".to_string(),
            })
        }
        Err(error) => {
            if trusted_answers.is_empty() {
                return None;
            }
            Some(DnsIntegrityFinding {
                domain: domain.to_string(),
                local_answers: Vec::new(),
                trusted_answers,
                local_error: Some(error),
                trusted_error_count,
                reason: "Local resolver failed while trusted public DNS succeeded".to_string(),
            })
        }
    }
}

fn answers_overlap(local: &[String], trusted: &[String]) -> bool {
    local.iter().any(|answer| trusted.contains(answer))
}

fn classify_integrity_findings(
    findings: Vec<DnsIntegrityFinding>,
    checked_domains: u8,
) -> DnsIntegrityStatus {
    let mismatch_count = findings.len() as u8;

    if mismatch_count == 0 {
        return DnsIntegrityStatus {
            state: DnsIntegrityState::Ok,
            confidence: DnsIntegrityConfidence::Low,
            mismatch_count: 0,
            checked_domains,
            summary: "DNS answers match trusted public resolvers".to_string(),
            details: Vec::new(),
        };
    }

    let local_failure_count = findings
        .iter()
        .filter(|finding| finding.local_error.is_some())
        .count() as u8;

    let confidence = if mismatch_count >= 2 && local_failure_count >= 2 {
        DnsIntegrityConfidence::High
    } else if mismatch_count >= 2 {
        DnsIntegrityConfidence::Medium
    } else {
        DnsIntegrityConfidence::Low
    };

    let state = match confidence {
        DnsIntegrityConfidence::Low => DnsIntegrityState::Caution,
        DnsIntegrityConfidence::Medium | DnsIntegrityConfidence::High => DnsIntegrityState::Suspicious,
    };

    let summary = match state {
        DnsIntegrityState::Ok => "DNS answers match trusted public resolvers".to_string(),
        DnsIntegrityState::Caution => format!(
            "Possible DNS inconsistency on {mismatch_count} of {checked_domains} checked domains"
        ),
        DnsIntegrityState::Suspicious => format!(
            "Likely DNS tampering or poisoning on {mismatch_count} of {checked_domains} checked domains"
        ),
    };

    DnsIntegrityStatus {
        state,
        confidence,
        mismatch_count,
        checked_domains,
        summary,
        details: findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(domain: &str, local_error: Option<&str>) -> DnsIntegrityFinding {
        DnsIntegrityFinding {
            domain: domain.to_string(),
            local_answers: if local_error.is_some() {
                Vec::new()
            } else {
                vec!["203.0.113.1".to_string()]
            },
            trusted_answers: vec!["198.51.100.1".to_string()],
            local_error: local_error.map(str::to_string),
            trusted_error_count: 0,
            reason: "test".to_string(),
        }
    }

    #[test]
    fn classifies_clean_results_as_ok() {
        let status = classify_integrity_findings(Vec::new(), 4);
        assert_eq!(status.state, DnsIntegrityState::Ok);
        assert_eq!(status.mismatch_count, 0);
    }

    #[test]
    fn classifies_single_mismatch_as_caution_low() {
        let status = classify_integrity_findings(vec![finding("example.com", None)], 4);
        assert_eq!(status.state, DnsIntegrityState::Caution);
        assert_eq!(status.confidence, DnsIntegrityConfidence::Low);
        assert_eq!(status.mismatch_count, 1);
    }

    #[test]
    fn classifies_multiple_mismatches_as_suspicious_medium() {
        let status = classify_integrity_findings(
            vec![finding("example.com", None), finding("github.com", None)],
            4,
        );
        assert_eq!(status.state, DnsIntegrityState::Suspicious);
        assert_eq!(status.confidence, DnsIntegrityConfidence::Medium);
    }

    #[test]
    fn classifies_repeated_local_failures_as_high() {
        let status = classify_integrity_findings(
            vec![
                finding("example.com", Some("timeout")),
                finding("github.com", Some("timeout")),
            ],
            4,
        );
        assert_eq!(status.confidence, DnsIntegrityConfidence::High);
        assert_eq!(status.state, DnsIntegrityState::Suspicious);
    }

    #[test]
    fn detects_answer_overlap() {
        assert!(answers_overlap(
            &["1.1.1.1".to_string()],
            &["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        ));
        assert!(!answers_overlap(
            &["203.0.113.1".to_string()],
            &["198.51.100.1".to_string()],
        ));
    }

    #[test]
    fn normalizes_domains() {
        let domains = normalize_verification_domains(&[
            " Example.COM ".to_string(),
            "github.com".to_string(),
            "example.com".to_string(),
        ])
        .expect("normalize");

        assert_eq!(domains, vec!["example.com", "github.com"]);
    }

    #[test]
    fn rejects_invalid_domains() {
        let error = normalize_verification_domains(&["not valid!".to_string()])
            .expect_err("invalid");
        assert!(error.to_string().contains("Invalid domain"));
    }

    #[test]
    fn default_settings_match_builtins() {
        let defaults = default_dns_integrity_settings();
        assert_eq!(
            defaults.verification_domains,
            DEFAULT_VERIFICATION_DOMAINS
                .iter()
                .map(|domain| (*domain).to_string())
                .collect::<Vec<_>>()
        );
    }
}
