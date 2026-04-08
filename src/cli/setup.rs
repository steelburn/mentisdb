use crate::integrations::apply::apply_setup_with_environment;
use crate::integrations::plan::{build_setup_plan_for_integration, SetupPlan};
use crate::integrations::IntegrationKind;
use crate::paths::{HostPlatform, PathEnvironment};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

use super::args::default_url;
use super::prompt::boxed_apply_summary;
use super::SetupCommand;

/// Minimum Node.js major version required by mcp-remote.
const MCP_REMOTE_MIN_NODE_MAJOR: u32 = 20;

pub(super) fn run_setup(
    command: &SetupCommand,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> io::Result<()> {
    let env = PathEnvironment::capture();
    let platform = HostPlatform::current();
    let mut plans = Vec::with_capacity(command.integrations.len());

    for integration in &command.integrations {
        let url = command
            .url
            .clone()
            .unwrap_or_else(|| default_url(*integration).to_string());
        let Some(plan) =
            build_setup_plan_for_integration(*integration, url.clone(), platform, &env)
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported integration target",
            ));
        };
        plans.push(plan);
    }

    for plan in &plans {
        write!(out, "{}", render_setup_plan(plan))?;
    }

    if command.dry_run {
        return Ok(());
    }

    let apply_items: Vec<(String, String)> = plans
        .iter()
        .map(|plan| {
            (
                plan.integration.display_name().to_owned(),
                plan.spec.config_target.path.display().to_string(),
            )
        })
        .collect();

    if !command.assume_yes {
        let response = boxed_apply_summary(out, &apply_items, true, input)?;
        if response.eq_ignore_ascii_case("n") || response.eq_ignore_ascii_case("no") {
            writeln!(out, "\nCancelled.")?;
            return Ok(());
        }
    }

    writeln!(out)?;
    for plan in plans {
        ensure_prerequisites(plan.integration, out)?;
        let result = apply_setup_with_environment(plan.integration, plan.url, platform, &env)?;
        writeln!(
            out,
            "{} -> {} ({})",
            plan.integration.display_name(),
            result.path.display(),
            if result.changed {
                "updated"
            } else {
                "unchanged"
            }
        )?;
    }

    Ok(())
}

/// Render a human-readable setup plan.
pub fn render_setup_plan(plan: &SetupPlan) -> String {
    let mut rendered = String::new();
    rendered.push_str("MentisDB setup plan\n\n");
    rendered.push_str(&format!(
        "Agent: {}\nPlatform: {}\nURL: {}\nTarget: {}\nStatus: {}\nAction: {}\n",
        plan.integration.display_name(),
        plan.platform.as_str(),
        plan.url,
        plan.spec.config_target.path.display(),
        plan.detection_status.as_str(),
        plan.action.as_str(),
    ));
    if let Some(command) = &plan.suggested_command {
        rendered.push_str(&format!("Command: {command}\n"));
    }
    if let Some(snippet) = &plan.snippet {
        rendered.push_str("\nExample config snippet:\n");
        rendered.push_str(snippet);
        rendered.push('\n');
    }
    if !plan.notes.is_empty() {
        rendered.push_str("\nNotes:\n");
        for note in &plan.notes {
            rendered.push_str("- ");
            rendered.push_str(note);
            rendered.push('\n');
        }
    }
    rendered.push('\n');
    rendered
}

pub(super) fn ensure_prerequisites(
    integration: IntegrationKind,
    out: &mut dyn Write,
) -> io::Result<()> {
    if integration != IntegrationKind::ClaudeDesktop {
        return Ok(());
    }

    if command_on_path(&["mcp-remote", "mcp-remote.cmd"]).is_some() {
        if let Some(node) = command_on_path(&["node", "node.exe"]) {
            match node_major_version(&node) {
                Ok(major) if major >= MCP_REMOTE_MIN_NODE_MAJOR => return Ok(()),
                Ok(major) => {
                    return Err(io::Error::other(format!(
                        "Claude Desktop requires Node.js >= {MCP_REMOTE_MIN_NODE_MAJOR} for mcp-remote, but {} is Node {major}. Install Node >= {MCP_REMOTE_MIN_NODE_MAJOR} or switch via nvm/fnm.",
                        node.display()
                    )));
                }
                Err(e) => {
                    return Err(io::Error::other(format!(
                        "Could not determine Node.js version from {}: {e}",
                        node.display()
                    )));
                }
            }
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Claude Desktop requires Node.js >= {MCP_REMOTE_MIN_NODE_MAJOR} for mcp-remote, but `node` was not found on PATH."),
        ));
    }

    let Some(npm) = command_on_path(&["npm", "npm.cmd"]) else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Claude Desktop integration requires npm so MentisDB can install mcp-remote.",
        ));
    };

    let Some(node) = command_on_path(&["node", "node.exe"]) else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Claude Desktop requires Node.js >= {MCP_REMOTE_MIN_NODE_MAJOR} for mcp-remote, but `node` was not found on PATH."),
        ));
    };

    match node_major_version(&node) {
        Ok(major) if major < MCP_REMOTE_MIN_NODE_MAJOR => {
            return Err(io::Error::other(format!(
                "Claude Desktop requires Node.js >= {MCP_REMOTE_MIN_NODE_MAJOR} for mcp-remote, but {} is Node {major}. Install Node >= {MCP_REMOTE_MIN_NODE_MAJOR} or switch via nvm/fnm.",
                node.display()
            )));
        }
        Ok(_) => {}
        Err(e) => {
            return Err(io::Error::other(format!(
                "Could not determine Node.js version from {}: {e}",
                node.display()
            )));
        }
    }

    writeln!(
        out,
        "Claude Desktop requires mcp-remote. Installing it with {}...",
        npm.display()
    )?;
    let status = Command::new(&npm)
        .args(["install", "-g", "mcp-remote"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "npm install -g mcp-remote failed with status {status}"
        )));
    }
    Ok(())
}

fn node_major_version(node: &PathBuf) -> io::Result<u32> {
    let output = Command::new(node).arg("--version").output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "node --version exited with status {}",
            output.status
        )));
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_node_major(&version)
}

/// Parse the major version number from a Node.js version string like `v22.18.0`.
pub fn parse_node_major(version: &str) -> io::Result<u32> {
    let version = version.trim_start_matches('v');
    let major_str = version
        .split('.')
        .next()
        .ok_or_else(|| io::Error::other(format!("unexpected node version format: {version}")))?;
    major_str.parse::<u32>().map_err(|e| {
        io::Error::other(format!(
            "could not parse node major version from {version}: {e}"
        ))
    })
}

fn command_on_path(candidates: &[&str]) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        for candidate in candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}
