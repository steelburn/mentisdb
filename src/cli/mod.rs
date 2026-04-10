//! Shared CLI helpers for `mentisdbd` setup, wizard, and memory subcommands.
//!
//! The daemon binary delegates subcommand parsing plus wizard/setup behavior to
//! this module so the command logic stays directly testable.

mod args;
mod prompt;
mod setup;
mod wizard;

pub use args::{
    parse_args, AddCommand, AgentsCommand, CliCommand, SearchCommand, SetupCommand, WizardCommand,
};
pub use prompt::{boxed_apply_summary, boxed_skip_notice, boxed_text_prompt, boxed_yn_prompt};
pub use setup::{parse_node_major, render_setup_plan};

use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::process::ExitCode;

/// Run the embedded CLI with caller-provided streams.
pub fn run_with_io<I, T>(
    args: I,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    match parse_args(args) {
        Ok(CliCommand::Help) => {
            let _ = write!(out, "{}", args::help_text());
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Setup(command)) => match setup::run_setup(&command, input, out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(err, "setup failed: {error}");
                ExitCode::from(1)
            }
        },
        Ok(CliCommand::Wizard(command)) => match wizard::run_wizard(&command, input, out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(err, "wizard failed: {error}");
                ExitCode::from(1)
            }
        },
        Ok(CliCommand::Add(command)) => match run_add(&command, out, err) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(err, "add failed: {error}");
                ExitCode::from(1)
            }
        },
        Ok(CliCommand::Search(command)) => match run_search(&command, out, err) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(err, "search failed: {error}");
                ExitCode::from(1)
            }
        },
        Ok(CliCommand::Agents(command)) => match run_agents(&command, out, err) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(err, "agents failed: {error}");
                ExitCode::from(1)
            }
        },
        Err(message) => {
            let _ = writeln!(err, "{message}");
            let _ = writeln!(err);
            let _ = write!(err, "{}", args::help_text());
            ExitCode::from(2)
        }
    }
}

fn run_add(cmd: &AddCommand, out: &mut dyn Write, _err: &mut dyn Write) -> Result<(), String> {
    let mut body = serde_json::Map::new();
    body.insert(
        "content".to_string(),
        serde_json::Value::String(cmd.content.clone()),
    );
    if let Some(ref thought_type) = cmd.thought_type {
        body.insert(
            "thought_type".to_string(),
            serde_json::Value::String(thought_type.clone()),
        );
    }
    if let Some(ref scope) = cmd.scope {
        body.insert(
            "scope".to_string(),
            serde_json::Value::String(scope.clone()),
        );
    }
    if !cmd.tags.is_empty() {
        body.insert(
            "tags".to_string(),
            serde_json::Value::Array(
                cmd.tags
                    .iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            ),
        );
    }
    if let Some(ref agent_id) = cmd.agent_id {
        body.insert(
            "agent_id".to_string(),
            serde_json::Value::String(agent_id.clone()),
        );
    }
    if let Some(ref chain_key) = cmd.chain_key {
        body.insert(
            "chain_key".to_string(),
            serde_json::Value::String(chain_key.clone()),
        );
    }
    let url = format!("{}/v1/thoughts", cmd.url.trim_end_matches('/'));
    let response = ureq::post(&url)
        .send_json(serde_json::Value::Object(body))
        .map_err(|e| format!("POST {url}: {e}"))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("parse response: {e}"))?;
    let _ = writeln!(
        out,
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    Ok(())
}

fn run_search(
    cmd: &SearchCommand,
    out: &mut dyn Write,
    _err: &mut dyn Write,
) -> Result<(), String> {
    let mut body = serde_json::Map::new();
    body.insert(
        "text".to_string(),
        serde_json::Value::String(cmd.text.clone()),
    );
    if let Some(limit) = cmd.limit {
        body.insert("limit".to_string(), serde_json::Value::Number(limit.into()));
    }
    if let Some(ref scope) = cmd.scope {
        body.insert(
            "scope".to_string(),
            serde_json::Value::String(scope.clone()),
        );
    }
    if let Some(ref chain_key) = cmd.chain_key {
        body.insert(
            "chain_key".to_string(),
            serde_json::Value::String(chain_key.clone()),
        );
    }
    let url = format!("{}/v1/ranked-search", cmd.url.trim_end_matches('/'));
    let response = ureq::post(&url)
        .send_json(serde_json::Value::Object(body))
        .map_err(|e| format!("POST {url}: {e}"))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("parse response: {e}"))?;
    let _ = writeln!(
        out,
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    Ok(())
}

fn run_agents(
    cmd: &AgentsCommand,
    out: &mut dyn Write,
    _err: &mut dyn Write,
) -> Result<(), String> {
    let mut url = format!("{}/v1/agents", cmd.url.trim_end_matches('/'));
    if let Some(ref chain_key) = cmd.chain_key {
        url = format!("{url}?chain_key={chain_key}");
    }
    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("parse response: {e}"))?;
    let _ = writeln!(
        out,
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    Ok(())
}
