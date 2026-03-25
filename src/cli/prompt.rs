//! Terminal prompt helpers with consistent boxed styling.
//!
//! All interactive prompts in the setup and wizard flows use a unified
//! bordered-box style so the TUI feels deliberate rather than ad-hoc.

use std::io::{self, BufRead, Write};

/// Colour escapes for prompt output.  Kept minimal so the module has no
/// extra feature-flagged dependencies.
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Render a multi-line boxed yes/no prompt and read the user's answer.
///
/// `question` is rendered inside the box.  The bottom row shows the choice
/// options with the default letter capitalised.
///
/// Returns the trimmed line the user typed, or an empty string on EOF.
pub fn boxed_yn_prompt(
    out: &mut dyn Write,
    question: &str,
    default_yes: bool,
    input: &mut dyn BufRead,
) -> io::Result<String> {
    let (yes_lbl, no_lbl, default_tag) = if default_yes {
        ("Y", "n", "Y")
    } else {
        ("y", "N", "N")
    };

    let choice_row = format!(" [{yes_lbl}] Yes  [{no_lbl}] No  (default: {default_tag}) ");
    let width = question
        .lines()
        .map(|l| l.len())
        .max()
        .unwrap_or(0)
        .max(choice_row.len());

    let border: String = format!("+{}+", "-".repeat(width + 2));

    writeln!(out)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    for line in question.lines() {
        let line_pad = " ".repeat(width.saturating_sub(line.len()));
        writeln!(out, "| {}{}{}{} |", BOLD, line, RESET, line_pad)?;
    }
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    let pad = " ".repeat(width.saturating_sub(choice_row.len()));
    writeln!(out, "| {}{}{}{}{} |", DIM, choice_row, RESET, pad, RESET)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    write!(out, "  > ")?;
    out.flush()?;
    read_line(input)
}

/// Render a summary box showing every integration that will be written,
/// then prompt for a final yes/no confirmation.
///
/// `items` is a slice of `(integration_display_name, path_display)` pairs.
pub fn boxed_apply_summary(
    out: &mut dyn Write,
    items: &[(String, String)],
    default_yes: bool,
    input: &mut dyn BufRead,
) -> io::Result<String> {
    let (yes_lbl, no_lbl, default_tag) = if default_yes {
        ("Y", "n", "Y")
    } else {
        ("y", "N", "N")
    };

    let header = " Apply these setup changes? ";
    let name_len = items.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    let path_len = items.iter().map(|(_, p)| p.len()).max().unwrap_or(0);
    let width = header.len().max(name_len + path_len + 6).max(50);

    let border: String = format!("+{}+", "-".repeat(width + 2));
    let choice_row = format!(" [{yes_lbl}] Apply  [{no_lbl}] Cancel  (default: {default_tag}) ");

    writeln!(out)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    let h_pad = " ".repeat(width.saturating_sub(header.len()));
    writeln!(out, "| {}{}{}{} |", BOLD, header, RESET, h_pad)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    for (name, path) in items {
        let n_pad = " ".repeat(name_len.saturating_sub(name.len()));
        let p_pad = " ".repeat(path_len.saturating_sub(path.len()));
        writeln!(
            out,
            "|   {}{}{}{}  {}{}{}{}  |",
            BOLD, name, RESET, n_pad, DIM, path, RESET, p_pad
        )?;
    }
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    let c_pad = " ".repeat(width.saturating_sub(choice_row.len()));
    writeln!(out, "| {}{}{}{} |", DIM, choice_row, RESET, c_pad)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    write!(out, "  > ")?;
    out.flush()?;
    read_line(input)
}

/// Render a compact single-question box and read the user's freeform answer.
/// Used for integration selection and URL-override prompts.
pub fn boxed_text_prompt(
    out: &mut dyn Write,
    question: &str,
    input: &mut dyn BufRead,
) -> io::Result<String> {
    let width = question.len().max(50);
    let border: String = format!("+{}+", "-".repeat(width + 2));

    writeln!(out)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    writeln!(out, "| {}{}{} |", BOLD, question, RESET)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    write!(out, "  > ")?;
    out.flush()?;
    read_line(input)
}

/// Render a single-line informational notice box for an integration that is
/// already configured and will be skipped.
pub fn boxed_skip_notice(out: &mut dyn Write, integration_name: &str) -> io::Result<()> {
    let msg = format!("{integration_name} already has a mentisdb integration — skipped.");
    let width = msg.len().max(60);
    let border: String = format!("+{}+", "-".repeat(width + 2));

    writeln!(out)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    writeln!(out, "{}| {} |{}", DIM, msg, RESET)?;
    writeln!(out, "{}{}{}", DIM, border, RESET)?;
    Ok(())
}

fn read_line(input: &mut dyn BufRead) -> io::Result<String> {
    let mut line = String::new();
    input.read_line(&mut line)?;
    Ok(line.trim_end_matches(&['\r', '\n'][..]).to_string())
}
