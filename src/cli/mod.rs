//! Shared CLI helpers for `mentisdbd` setup and wizard subcommands.
//!
//! The daemon binary delegates subcommand parsing plus wizard/setup behavior to
//! this module so the command logic stays directly testable.

mod args;
mod prompt;
mod setup;
mod wizard;

pub use args::{parse_args, CliCommand, SetupCommand, WizardCommand};
pub use prompt::{boxed_apply_summary, boxed_skip_notice, boxed_text_prompt, boxed_yn_prompt};
pub use setup::render_setup_plan;

use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::process::ExitCode;

/// Run the embedded setup/wizard CLI with caller-provided streams.
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
        Ok(CliCommand::Setup(command)) => match setup::run_setup(&command, out) {
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
        Err(message) => {
            let _ = writeln!(err, "{message}");
            let _ = writeln!(err);
            let _ = write!(err, "{}", args::help_text());
            ExitCode::from(2)
        }
    }
}
