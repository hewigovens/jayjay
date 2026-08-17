use std::path::Path;

use jj_lib::workspace::{DefaultWorkspaceLoaderFactory, WorkspaceLoaderFactory as _};

pub fn workspace_primary_root(path: &str) -> Option<String> {
    let loader = DefaultWorkspaceLoaderFactory.create(Path::new(path)).ok()?;
    let repo_dir = loader.repo_path();
    if !repo_dir.is_dir() {
        return None;
    }
    let root = dunce::canonicalize(repo_dir.parent()?.parent()?).ok()?;
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
