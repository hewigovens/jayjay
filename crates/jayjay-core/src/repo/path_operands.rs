//! Safe encodings of repository-controlled paths for jj command operands and
//! `.gitignore` lines, so an attacker-chosen filename can't act as an option, a
//! fileset expression, or an extra ignore rule.

use crate::types::*;

/// Wrap a repo-relative path as an exact-match jj fileset operand, so a filename with
/// fileset syntax (`all()`, `glob:`) or a leading `-` is matched literally, not evaluated
/// as an expression or parsed as an option. `root-file:` is repo-root-relative (cwd-independent).
pub(crate) fn fileset_literal(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len() + 2);
    for ch in path.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(ch),
        }
    }
    format!("root-file:\"{escaped}\"")
}

/// Reject paths with control characters: a newline in a filename would otherwise
/// inject extra `.gitignore` patterns.
pub(crate) fn reject_control_chars(paths: &[String]) -> CoreResult<()> {
    for path in paths {
        if path.chars().any(char::is_control) {
            return Err(CoreError::Internal {
                message: format!("path contains control characters: {path:?}"),
            });
        }
    }
    Ok(())
}

/// Escape a path into a literal `.gitignore` pattern: leading `!`/`#` and glob
/// metacharacters `\ * ? [ ]` are escaped so the file is ignored verbatim, never a
/// broader or inverted rule. Reject control chars first — gitignore has no newline escape.
pub(crate) fn gitignore_pattern(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 1);
    for (i, ch) in path.chars().enumerate() {
        match ch {
            '\\' | '*' | '?' | '[' | ']' => {
                out.push('\\');
                out.push(ch);
            }
            '!' | '#' if i == 0 => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fileset_literal_wraps_and_escapes() {
        assert_eq!(fileset_literal("a/b.txt"), "root-file:\"a/b.txt\"");
        assert_eq!(fileset_literal("all()"), "root-file:\"all()\"");
        assert_eq!(fileset_literal("--config=x"), "root-file:\"--config=x\"");
        assert_eq!(fileset_literal("a\"b"), "root-file:\"a\\\"b\"");
        assert_eq!(fileset_literal("a\\b"), "root-file:\"a\\\\b\"");
    }

    #[test]
    fn reject_control_chars_flags_newline_and_tab() {
        assert!(reject_control_chars(&["ok/path.txt".to_owned()]).is_ok());
        assert!(reject_control_chars(&["evil\n*.pem".to_owned()]).is_err());
        assert!(reject_control_chars(&["tab\there".to_owned()]).is_err());
    }

    #[test]
    fn gitignore_pattern_escapes_special_leading_and_glob() {
        assert_eq!(gitignore_pattern("normal.txt"), "normal.txt");
        assert_eq!(gitignore_pattern("!important"), "\\!important");
        assert_eq!(gitignore_pattern("#hash"), "\\#hash");
        assert_eq!(gitignore_pattern("a*b?.txt"), "a\\*b\\?.txt");
        assert_eq!(gitignore_pattern("x[y].txt"), "x\\[y\\].txt");
        // `!` is only special at the start.
        assert_eq!(gitignore_pattern("a!b"), "a!b");
    }
}
