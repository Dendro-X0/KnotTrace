//! Headless CLI args for `network-probe` (`check --json`).

use crate::CheckProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliOutput {
    Report,
    Explain,
    Bundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonStyle {
    Compact,
    Pretty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Help,
    Check {
        profile: CheckProfile,
        output: CliOutput,
        style: JsonStyle,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn usage() -> &'static str {
    "KnotTrace network-probe — headless Path Truth check

Usage:
  network-probe check --json [--fast|--full] [--explain|--bundle]
  network-probe check --pretty [--fast|--full] [--explain|--bundle]
  network-probe [--explain]
  network-probe --help

Flags:
  check           Run a health check (preferred form for agents)
  --json          Compact JSON on stdout
  --pretty        Indented JSON (default for `check` when style omitted)
  --fast          Fast check profile
  --full          Full check profile (default)
  --explain       Emit local explain JSON instead of HealthReport
  --bundle        Emit redacted diagnostic bundle JSON instead of HealthReport
  --help, -h      Show this help

Exit codes: 0 ok · 1 check failed · 2 usage error

Legacy: bare `network-probe` still prints a pretty HealthReport (same as `check --pretty`).
"
}

/// Parse argv. Accepts full argv (with binary name) or args after the binary.
pub fn parse_args<I, S>(args: I) -> Result<CliCommand, ParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let tokens: Vec<String> = args
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect();

    let rest = strip_program_name(&tokens);

    if rest.is_empty() {
        return Ok(legacy_pretty_report());
    }

    if rest.iter().any(|t| t == "--help" || t == "-h" || t == "help") {
        return Ok(CliCommand::Help);
    }

    if rest[0] == "check" {
        return parse_check_flags(&rest[1..]);
    }

    // Legacy: only `--explain` (repeatable/single) allowed without `check`.
    if rest.iter().all(|t| t == "--explain") {
        return Ok(CliCommand::Check {
            profile: CheckProfile::Full,
            output: CliOutput::Explain,
            style: JsonStyle::Pretty,
        });
    }

    Err(ParseError::new(format!(
        "unknown argument: {} (try `network-probe check --json` or --help)",
        rest[0]
    )))
}

fn strip_program_name(tokens: &[String]) -> Vec<String> {
    match tokens.first().map(String::as_str) {
        Some(first)
            if !first.starts_with('-')
                && first != "check"
                && first != "help"
                && !first.is_empty() =>
        {
            tokens[1..].to_vec()
        }
        _ => tokens.to_vec(),
    }
}

fn legacy_pretty_report() -> CliCommand {
    CliCommand::Check {
        profile: CheckProfile::Full,
        output: CliOutput::Report,
        style: JsonStyle::Pretty,
    }
}

fn parse_check_flags(flags: &[String]) -> Result<CliCommand, ParseError> {
    let mut profile = CheckProfile::Full;
    let mut output = CliOutput::Report;
    let mut style = JsonStyle::Pretty;
    let mut saw_explain = false;
    let mut saw_bundle = false;

    for tok in flags {
        match tok.as_str() {
            "--json" => style = JsonStyle::Compact,
            "--pretty" => style = JsonStyle::Pretty,
            "--fast" => profile = CheckProfile::Fast,
            "--full" => profile = CheckProfile::Full,
            "--explain" => {
                saw_explain = true;
                output = CliOutput::Explain;
            }
            "--bundle" => {
                saw_bundle = true;
                output = CliOutput::Bundle;
            }
            other => {
                return Err(ParseError::new(format!("unknown argument: {other}")));
            }
        }
    }

    if saw_explain && saw_bundle {
        return Err(ParseError::new(
            "--explain and --bundle are mutually exclusive",
        ));
    }

    Ok(CliCommand::Check {
        profile,
        output,
        style,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_invoke_is_pretty_report() {
        assert_eq!(parse_args(["network-probe"]).unwrap(), legacy_pretty_report());
        assert_eq!(parse_args(Vec::<&str>::new()).unwrap(), legacy_pretty_report());
    }

    #[test]
    fn legacy_explain() {
        assert_eq!(
            parse_args(["network-probe", "--explain"]).unwrap(),
            CliCommand::Check {
                profile: CheckProfile::Full,
                output: CliOutput::Explain,
                style: JsonStyle::Pretty,
            }
        );
    }

    #[test]
    fn check_json_fast_bundle() {
        assert_eq!(
            parse_args(["network-probe", "check", "--json", "--fast", "--bundle"]).unwrap(),
            CliCommand::Check {
                profile: CheckProfile::Fast,
                output: CliOutput::Bundle,
                style: JsonStyle::Compact,
            }
        );
    }

    #[test]
    fn check_json_explain() {
        assert_eq!(
            parse_args(["network-probe", "check", "--json", "--explain"]).unwrap(),
            CliCommand::Check {
                profile: CheckProfile::Full,
                output: CliOutput::Explain,
                style: JsonStyle::Compact,
            }
        );
    }

    #[test]
    fn check_defaults_pretty_full_report() {
        assert_eq!(
            parse_args(["check"]).unwrap(),
            CliCommand::Check {
                profile: CheckProfile::Full,
                output: CliOutput::Report,
                style: JsonStyle::Pretty,
            }
        );
    }

    #[test]
    fn mutual_exclusion() {
        let err = parse_args(["check", "--explain", "--bundle"]).unwrap_err();
        assert!(err.message.contains("mutually exclusive"));
    }

    #[test]
    fn json_without_check_errors() {
        let err = parse_args(["network-probe", "--json"]).unwrap_err();
        assert!(err.message.contains("check"));
    }

    #[test]
    fn help() {
        assert_eq!(
            parse_args(["network-probe", "--help"]).unwrap(),
            CliCommand::Help
        );
        assert_eq!(parse_args(["check", "-h"]).unwrap(), CliCommand::Help);
    }
}
