/// The primary workspace root backing the jj workspace at `path`, or None when `path` is not a jj workspace. Secondary workspaces store the primary's repo location in a `.jj/repo` file (relative to `.jj/`), primaries have a `.jj/repo` directory.
pub fn workspace_primary_root(path: &str) -> Option<String> {
    let jj_dir = std::path::Path::new(path).join(".jj");
    let marker = jj_dir.join("repo");
    let metadata = std::fs::metadata(&marker).ok()?;
    if metadata.is_dir() {
        return Some(path.to_owned());
    }
    let contents = std::fs::read_to_string(&marker).ok()?;
    let store = jj_dir.join(contents.trim());
    let root = std::fs::canonicalize(store.parent()?.parent()?).ok()?;
    Some(root.to_string_lossy().into_owned())
}

/// True when `name` is safe both as a jj workspace name and as the sibling directory both shells create for it: no path separators or traversal, no option shape, no characters that are invalid in directory names on any supported platform.
pub fn is_valid_workspace_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 || name == "." || name == ".." {
        return false;
    }
    if name.starts_with('-') || name.ends_with('.') {
        return false;
    }
    !name.chars().any(|ch| {
        ch.is_control()
            || ch.is_whitespace()
            || matches!(
                ch,
                '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*' | '@'
            )
    })
}

#[cfg(test)]
mod tests {
    use super::is_valid_workspace_name;

    #[test]
    fn workspace_names_accept_simple_directory_safe_names() {
        for ok in ["feature", "feature-2", "a_b", "ws.1", "Über"] {
            assert!(is_valid_workspace_name(ok), "{ok} should be valid");
        }
    }

    #[test]
    fn workspace_names_reject_path_option_and_revset_shapes() {
        let bad = [
            "",
            ".",
            "..",
            "-x",
            "--name",
            "a/b",
            "a\\b",
            "../up",
            "a b",
            "a\tb",
            "a\nb",
            "a@b",
            "@",
            "a:b",
            "a*b",
            "a?b",
            "trailing.",
        ];
        for name in bad {
            assert!(!is_valid_workspace_name(name), "{name:?} should be invalid");
        }
    }
}
