#![forbid(unsafe_code)]

use std::io;

use ores_clis_core::{ColorRole, EmitDisposition, ProtocolEmitter, StreamRole, paint, top_level_io};

use crate::config::Config;
use crate::error::CliError;

pub fn run(config: &Config) -> Result<(), CliError> {
    let body = serde_json::json!({
        "service": "declmig",
        "api_base": config.api_base,
    });
    let stdout = io::stdout();
    let mut output = ProtocolEmitter::new(stdout.lock(), StreamRole::Primary);
    let write = if config.json {
        output.emit_primary_machine_record(&body.to_string())
    } else {
        let line = format!(
            "{} @ {}",
            paint(config.runtime.color_stdout(), ColorRole::Emphasis, "declmig"),
            paint(config.runtime.color_stdout(), ColorRole::Info, &config.api_base)
        );
        output.emit_primary_human_line(&line)
    };
    match top_level_io(write)
        .map_err(|error| CliError::Command(format!("stdout write failed: {error}")))?
    {
        EmitDisposition::Written | EmitDisposition::ConsumerClosed => Ok(()),
    }
}
