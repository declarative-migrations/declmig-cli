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
        // Preserve the existing environment override, but explicit argv wins.
        if !invocation.output_was_explicit && std::env::var_os("DECLMIG_JSON").is_some() {
            policy.output = OutputMode::Json;
        }
        let runtime = policy.resolve(TerminalState::detect(), EnvironmentHints::detect());

        Ok(Self {
            api_base,
            json: runtime.json(),
            runtime,
        })
    }
}
