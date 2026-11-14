#![forbid(unsafe_code)]

use ores_clis_core::{parse_shared_argv, CliPolicy};

use crate::error::CliError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Health,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command: Command,
    pub api_base: Option<String>,
    pub policy: CliPolicy,
    pub output_was_explicit: bool,
}

pub fn parse<I>(args: I) -> Result<Invocation, CliError>
where
    I: IntoIterator<Item = String>,
{
    let shared = parse_shared_argv(args).map_err(|error| CliError::Usage(error.to_string()))?;
    let mut command = Command::Help;
    let mut api_base = None;
    let output_was_explicit = shared.output_was_explicit();
    let mut items = shared.passthrough.into_iter().peekable();
    if let Some(first) = items.peek() {
        match first.as_str() {
            "health" => {
                command = Command::Health;
                items.next();
            }
            "status" => {
                command = Command::Status;
                items.next();
            }
            "-h" | "--help" | "help" => {
                command = Command::Help;
                items.next();
            }
            other if !other.starts_with('-') => {
                return Err(CliError::Usage(format!("unknown command {other}")));
            }
            _ => {}
        }
    }
    for arg in items {
        match arg.as_str() {
            "--help" | "-h" => command = Command::Help,
            flag if flag.starts_with("--api-base=") => {
                if api_base.is_some() {
                    return Err(CliError::Usage("--api-base may appear only once".into()));
                }
                let value = flag.trim_start_matches("--api-base=");
                if !valid_api_base(value) {
                    return Err(CliError::Usage(
                        "--api-base must be an http(s) URL without credentials, query, or fragment"
                            .into(),
                    ));
                }
                api_base = Some(value.to_string());
            }
            other => return Err(CliError::Usage(format!("unknown flag {other}"))),
        }
    }
    Ok(Invocation {
        command,
        api_base,
        policy: shared.policy,
        output_was_explicit,
    })
}

fn valid_api_base(value: &str) -> bool {
    let rest = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"));
    let Some(rest) = rest else { return false };
    let authority = rest.split('/').next().unwrap_or("");
    !authority.is_empty()
        && !authority.contains('@')
        && !rest.contains('?')
        && !rest.contains('#')
        && !rest.contains('\\')
        && !rest.chars().any(char::is_whitespace)
}

pub fn help_text() -> &'static str {
    "declmig — Declarative Migrations CLI\n\nCommands:\n  health\n  status\n\nShared output flags:\n  --json / --no-json\n  --output=auto|human|json\n  --color / --no-color\n  --log-level=silent|quiet|error|warn|info|debug|trace\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use ores_clis_core::{ColorMode, LogLevel, OutputMode};

    #[test]
    fn shared_flags_are_removed_before_domain_parsing() {
        let invocation = parse([
            "health".to_owned(),
            "--color".to_owned(),
            "--log-level=trace".to_owned(),
            "--json".to_owned(),
            "--api-base=https://example.test".to_owned(),
        ])
        .unwrap();
        assert_eq!(invocation.command, Command::Health);
        assert_eq!(invocation.api_base.as_deref(), Some("https://example.test"));
        assert_eq!(invocation.policy.color, ColorMode::Always);
        assert_eq!(invocation.policy.log_level, LogLevel::Trace);
        assert_eq!(invocation.policy.output, OutputMode::Json);
        assert!(invocation.output_was_explicit);
    }

    #[test]
    fn api_base_rejects_ambiguous_or_unsafe_values() {
        for value in [
            "",
            "localhost:8080",
            "https://",
            "https://user:pass@example.test",
            "https://example.test/?q=1",
            "https://example.test/#anchor",
            "https://example.test/with space",
        ] {
            assert!(
                parse(["health".to_owned(), format!("--api-base={value}")]).is_err(),
                "{value}"
            );
        }
        assert!(parse([
            "status".to_owned(),
            "--api-base=https://example.test".to_owned(),
            "--api-base=http://localhost:8080".to_owned()
        ])
        .is_err());
        assert!(parse([
            "health".to_owned(),
            "--api-base=http://localhost:8080".to_owned()
        ])
        .is_ok());
    }

    #[test]
    fn contradictory_shared_flags_fail_closed() {
        assert!(parse([
            "status".to_owned(),
            "--json".to_owned(),
            "--no-json".to_owned()
        ])
        .is_err());
    }
}
