use super::Repo;
use crate::types::*;

const IGNORE_WORKING_COPY_ARG: &str = "--ignore-working-copy";
const WORKSPACE_COMMAND: &str = "workspace";

impl Repo {
    /// List all workspaces for this repo.
    pub fn workspace_list(&self) -> CoreResult<Vec<WorkspaceInfo>> {
        let output = self.run_jj(&[IGNORE_WORKING_COPY_ARG, WORKSPACE_COMMAND, "list"])?;
        let current_name = self.workspace_name.as_str();
        let mut workspaces = Vec::new();

        for line in output.lines() {
            let name = line.split(':').next().unwrap_or("").trim().to_owned();
            if name.is_empty() {
                continue;
            }
            let path = self
                .run_jj(&[
                    IGNORE_WORKING_COPY_ARG,
                    WORKSPACE_COMMAND,
                    "root",
                    "--name",
                    &name,
                ])
                .unwrap_or_default();
            let is_current = name == current_name;
            workspaces.push(WorkspaceInfo {
                name,
                path,
                is_current,
            });
        }

        Ok(workspaces)
    }

    /// Create a new workspace at the given path, optionally on a specific revision.
    pub fn workspace_add(&self, dest: &str, name: &str, rev: &str) -> CoreResult<String> {
        if !name.is_empty() && !is_valid_workspace_name(name) {
            return Err(CoreError::Internal {
                message: format!("invalid workspace name: {name}"),
            });
        }
        if rev.starts_with('-') {
            return Err(CoreError::Internal {
                message: format!("invalid revision: {rev}"),
            });
        }
        let mut args = vec![WORKSPACE_COMMAND, "add"];
        if !name.is_empty() {
            args.extend(["--name", name]);
        }
        if !rev.is_empty() {
            args.extend(["-r", rev]);
        }
        // `--` so an option-shaped destination is read as a literal path, never as a jj flag.
        args.extend(["--", dest]);
        let output = self.run_jj(&args)?;
        self.reload()?;
        Ok(output)
    }

    /// Remove a workspace.
    pub fn workspace_forget(&self, name: &str) -> CoreResult<()> {
        // `--` so an option-shaped workspace name is read as an operand, never as a jj flag.
        self.run_jj_reload(&[WORKSPACE_COMMAND, "forget", "--", name])
    }
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
