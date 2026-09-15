#![forbid(unsafe_code)]

use ores_clis_core::{EnvironmentHints, OutputMode, RuntimePolicy, TerminalState};

use crate::args::Invocation;
use crate::error::CliError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub api_base: String,
    pub json: bool,
    pub runtime: RuntimePolicy,
}

impl Config {
    pub fn load(invocation: &Invocation) -> Result<Self, CliError> {
        let api_base = invocation
            .api_base
            .clone()
            .or_else(|| std::env::var("DECLMIG_API_BASE").ok())
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());
        if api_base.trim().is_empty() {
            return Err(CliError::Config("API base is empty".into()));
        }

        let mut policy = invocation.policy;
        // Preserve the legacy environment override, but explicit shared argv wins.
        if !invocation.output_was_explicit {
            if let Some(json) = legacy_json_override("DECLMIG_JSON")? {
                policy.output = if json {
                    OutputMode::Json
                } else {
                    OutputMode::Human
                };
            }
        }
        let runtime = policy.resolve(TerminalState::detect(), EnvironmentHints::detect());

        Ok(Self {
            api_base,
            json: runtime.json(),
            runtime,
        })
    }
}

fn legacy_json_override(name: &str) -> Result<Option<bool>, CliError> {
    match std::env::var(name) {
        Ok(value) => parse_json_override(&value).map(Some),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(CliError::Config(format!(
            "{name} must be valid UTF-8"
        ))),
    }
}

fn parse_json_override(value: &str) -> Result<bool, CliError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(CliError::Config(
            "DECLMIG_JSON must be one of true/false, 1/0, yes/no, or on/off".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_json_override;

    #[test]
    fn legacy_json_override_is_tristate_capable() {
        for value in ["true", "1", "yes", "on", "TRUE"] {
            assert!(parse_json_override(value).unwrap());
        }
        for value in ["false", "0", "no", "off", "FALSE"] {
            assert!(!parse_json_override(value).unwrap());
        }
        assert!(parse_json_override("maybe").is_err());
    }
}
