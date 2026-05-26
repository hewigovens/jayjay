use std::path::Path;
use std::process::Command;

use crate::{CoreError, CoreResult, jj_binary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JjCommand {
    raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JjCommandResult {
    pub output: String,
    pub exit_code: i32,
}

impl JjCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            raw: command.into(),
        }
    }

    pub fn from_palette_query(query: &str) -> Option<Self> {
        let body_after = |rest: &str| Self::new(rest.trim_start());
        if query == "jj" || query == "!" {
            return Some(Self::new(String::new()));
        }
        if let Some(rest) = query.strip_prefix("jj ") {
            return Some(body_after(rest));
        }
        query.strip_prefix('!').map(body_after)
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn into_raw(self) -> String {
        self.raw
    }

    pub fn parse_args(&self) -> Option<Vec<String>> {
        parse_args(&self.raw)
    }

    pub fn run_in_path(&self, path: &Path) -> CoreResult<JjCommandResult> {
        let args = self.parse_args().ok_or_else(|| CoreError::Internal {
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

        Ok(JjCommandResult {
            output: combined,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}

fn parse_args(command: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaping = false;
    let mut arg_started = false;

    let mut chars = command.chars().peekable();
    while let Some(ch) = chars.next() {
        if escaping {
            current.push(ch);
            escaping = false;
            arg_started = true;
            continue;
        }
        if let Some(current_quote) = quote {
            if ch == current_quote {
                quote = None;
            } else if current_quote == '"' && ch == '\\' {
                match chars.peek().copied() {
                    Some('"') | Some('\\') => current.push(chars.next().expect("peeked next char")),
                    _ => current.push(ch),
                }
            } else {
                current.push(ch);
            }
            arg_started = true;
            continue;
        }
        if ch == '\\' {
            escaping = true;
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

fn trim_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shell_like_args() {
        assert_eq!(
            JjCommand::new(r#"log -r "description(exact:'fix bug')" --limit 5"#).parse_args(),
            Some(vec![
                "log".to_owned(),
                "-r".to_owned(),
                "description(exact:'fix bug')".to_owned(),
                "--limit".to_owned(),
                "5".to_owned()
            ])
        );
        assert_eq!(
            JjCommand::new(r#"file\ with\ spaces"#).parse_args(),
            Some(vec!["file with spaces".to_owned()])
        );
        assert_eq!(
            JjCommand::new(r#"describe -m "a\"b""#).parse_args(),
            Some(vec![
                "describe".to_owned(),
                "-m".to_owned(),
                "a\"b".to_owned()
            ])
        );
        assert_eq!(
            JjCommand::new(r#"describe -m """#).parse_args(),
            Some(vec!["describe".to_owned(), "-m".to_owned(), String::new()])
        );
        assert_eq!(
            JjCommand::new(r#"new '' file\ with\ spaces"#).parse_args(),
            Some(vec![
                "new".to_owned(),
                String::new(),
                "file with spaces".to_owned()
            ])
        );
        assert_eq!(
            JjCommand::new(r#"describe -m 'a\b'"#).parse_args(),
            Some(vec![
                "describe".to_owned(),
                "-m".to_owned(),
                r#"a\b"#.to_owned()
            ])
        );
        assert_eq!(
            JjCommand::new(r#"log -r "description(regex:'\bfix\b')""#).parse_args(),
            Some(vec![
                "log".to_owned(),
                "-r".to_owned(),
                r#"description(regex:'\bfix\b')"#.to_owned()
            ])
        );
        assert_eq!(JjCommand::new(r#"log -r "mine()"#).parse_args(), None);
    }

    #[test]
    fn extracts_prefixed_command_body() {
        assert_eq!(
            JjCommand::from_palette_query("jj log").map(JjCommand::into_raw),
            Some("log".to_owned())
        );
        assert_eq!(
            JjCommand::from_palette_query("!status").map(JjCommand::into_raw),
            Some("status".to_owned())
        );
        assert_eq!(JjCommand::from_palette_query("status"), None);
    }
}
