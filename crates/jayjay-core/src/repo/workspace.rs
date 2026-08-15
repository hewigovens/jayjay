use std::path::Path;

use jj_lib::object_id::ObjectId;
use jj_lib::ref_name::WorkspaceName;
use jj_lib::repo::Repo as _;

use super::Repo;
use crate::types::*;

const IGNORE_WORKING_COPY_ARG: &str = "--ignore-working-copy";
const WORKSPACE_COMMAND: &str = "workspace";

impl Repo {
    /// List all workspaces for this repo.
    ///
    /// Names and paths come from `jj workspace list/root --ignore-working-copy`.
    /// Timestamp, description, and committed file counts are filled from this
    /// repo's in-memory view (`get_wc_commit_id`) so other working copies are
    /// never opened or snapshotted.
    pub fn workspace_list(&self) -> CoreResult<Vec<WorkspaceInfo>> {
        // Pick up sibling workspace describes/commits from the shared op log without snapshotting this WC.
        self.reload()?;
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
            workspaces.push(self.enrich_workspace_info(name, path, is_current));
        }

        WorkspaceInfo::sort_for_sidebar(&mut workspaces, current_name);
        Ok(workspaces)
    }

    /// Display name of `trunk()` when it resolves to exactly one bookmark.
    pub fn trunk_bookmark_name(&self) -> Option<String> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, "trunk()").ok()?;
        let names: Vec<String> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        match names.as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }

    /// Parent and working-copy commit ids for Show Changes. Never uses bare `@`.
    pub fn workspace_compare_revs(&self, name: &str) -> Option<(String, String)> {
        let info = self.workspace_info_metadata(name)?;
        if info.wc_commit_id.is_empty() || info.parent_commit_id.is_empty() {
            return None;
        }
        Some((info.parent_commit_id, info.wc_commit_id))
    }

    /// Diff of that workspace's working-copy commit versus its first parent. Uses the commit id, never bare `@` and never `edit`.
    pub fn workspace_show_changes(&self, name: &str) -> CoreResult<ChangeDetail> {
        self.reload()?;
        let Some(info) = self.workspace_info_metadata(name) else {
            return Err(CoreError::Internal {
                message: format!("workspace {name} has no working-copy commit to inspect"),
            });
        };
        if info.wc_commit_id.is_empty() {
            return Err(CoreError::Internal {
                message: format!("workspace {name} has no working-copy commit to inspect"),
            });
        }
        self.show_summary(&info.wc_commit_id)
    }

    fn enrich_workspace_info(&self, name: String, path: String, is_current: bool) -> WorkspaceInfo {
        let path_exists = Path::new(&path).is_dir();
        let Some(metadata) = self.workspace_info_metadata(&name) else {
            let mut info = WorkspaceInfo::new(name, path, is_current);
            info.path_exists = path_exists;
            return info;
        };
        let mut info = metadata;
        info.path = path;
        info.is_current = is_current;
        info.path_exists = path_exists;
        info
    }

    fn workspace_info_metadata(&self, name: &str) -> Option<WorkspaceInfo> {
        let repo = self.get_repo();
        let workspace_name = WorkspaceName::new(name);
        let commit_id = repo.view().get_wc_commit_id(workspace_name)?;
        let commit = repo.store().get_commit(commit_id).ok()?;
        let description = commit
            .description()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_owned();
        let parent_commit_id = commit
            .parent_ids()
            .first()
            .map(|id| id.hex())
            .unwrap_or_default();
        let changed_file_count = self.committed_file_count_vs_first_parent(&commit).ok();
        Some(WorkspaceInfo {
            name: name.to_owned(),
            path: String::new(),
            is_current: false,
            wc_commit_id: commit.id().hex(),
            parent_commit_id,
            timestamp_millis: Some(commit.author().timestamp.timestamp.0),
            changed_file_count,
            description,
            path_exists: false,
        })
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
