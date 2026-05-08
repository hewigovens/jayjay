use std::path::Path;
use std::process::Command;

use crate::{CoreError, CoreResult, JjCommandRun, jj_binary};

pub fn jj_command_body(query: &str) -> Option<String> {
    let body_after = |rest: &str| rest.trim_start().to_owned();
    if query == "jj" || query == "!" {
        return Some(String::new());
    }
    if let Some(rest) = query.strip_prefix("jj ") {
        return Some(body_after(rest));
    }
    query.strip_prefix('!').map(body_after)
}

pub fn parse_jj_command_args(command: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaping = false;
    let mut arg_started = false;

    for ch in command.chars() {
        if escaping {
            current.push(ch);
            escaping = false;
            arg_started = true;
            continue;
        }
        if ch == '\\' {
            escaping = true;
            arg_started = true;
            continue;
        }
        if let Some(current_quote) = quote {
            if ch == current_quote {
                quote = None;
            } else {
                current.push(ch);
            }
            arg_started = true;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            arg_started = true;
            continue;
        }
        if ch.is_whitespace() {
            if arg_started {
                args.push(std::mem::take(&mut current));
                arg_started = false;
            }
            continue;
        }
        current.push(ch);
        arg_started = true;
    }

    if escaping {
        current.push('\\');
    }
    quote.is_none().then(|| {
        if arg_started {
            args.push(current);
        }
        args
    })
}

pub fn record_jj_command_history(command: &str, existing: &[String], limit: usize) -> Vec<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return existing.to_vec();
    }

    let mut values = Vec::with_capacity(existing.len().min(limit).saturating_add(1));
    values.push(trimmed.to_owned());
    values.extend(
        existing
            .iter()
            .filter(|item| item.as_str() != trimmed)
            .cloned(),
    );
    values.truncate(limit);
    values
}

pub fn run_jj_command_in_path(path: &Path, command: &str) -> CoreResult<JjCommandRun> {
    let args = parse_jj_command_args(command).ok_or_else(|| CoreError::Internal {
        message: "Unclosed quote in jj command.".to_owned(),
    })?;
    if args.is_empty() {
        return Err(CoreError::Internal {
            message: "No jj command to run.".to_owned(),
        });
    }

    let output = Command::new(jj_binary())
        .args(&args)
        .current_dir(path)
        .output()
        .map_err(|e| CoreError::Internal {
            message: format!("run jj: {e}"),
        })?;

    let stdout = trim_output(&output.stdout);
    let stderr = trim_output(&output.stderr);
    let combined = match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "(no output)".to_owned(),
        (false, true) => stdout.clone(),
        (true, false) => stderr.clone(),
        (false, false) => format!("{stdout}\n{stderr}"),
    };

    Ok(JjCommandRun {
        display: format!("jj {command}"),
        stdout,
        stderr,
        output: combined,
        exit_code: output.status.code().unwrap_or(-1),
        success: output.status.success(),
    })
}

fn trim_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shell_like_args() {
        assert_eq!(
            parse_jj_command_args(r#"log -r "description(exact:'fix bug')" --limit 5"#),
            Some(vec![
                "log".to_owned(),
                "-r".to_owned(),
                "description(exact:'fix bug')".to_owned(),
                "--limit".to_owned(),
                "5".to_owned()
            ])
        );
        assert_eq!(
            parse_jj_command_args(r#"file\ with\ spaces"#),
            Some(vec!["file with spaces".to_owned()])
        );
        assert_eq!(
            parse_jj_command_args(r#"describe -m "a\"b""#),
            Some(vec![
                "describe".to_owned(),
                "-m".to_owned(),
                "a\"b".to_owned()
            ])
        );
        assert_eq!(
            parse_jj_command_args(r#"file\"#),
            Some(vec!["file\\".to_owned()])
        );
        assert_eq!(
            parse_jj_command_args(r#"describe -m """#),
            Some(vec!["describe".to_owned(), "-m".to_owned(), String::new()])
        );
        assert_eq!(
            parse_jj_command_args(r#"new '' file\ with\ spaces"#),
            Some(vec![
                "new".to_owned(),
                String::new(),
                "file with spaces".to_owned()
            ])
        );
        assert_eq!(parse_jj_command_args(r#"log -r "mine()"#), None);
    }

    #[test]
    fn extracts_prefixed_command_body() {
        assert_eq!(jj_command_body("jj log"), Some("log".to_owned()));
        assert_eq!(jj_command_body("!status"), Some("status".to_owned()));
        assert_eq!(jj_command_body("status"), None);
    }

    #[test]
    fn records_history_at_front_without_duplicates() {
        let existing = vec!["status".to_owned(), "log".to_owned()];
        assert_eq!(
            record_jj_command_history(" log ", &existing, 20),
            vec!["log".to_owned(), "status".to_owned()]
        );
    }
}
