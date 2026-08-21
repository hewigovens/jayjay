use std::path::{Path, PathBuf};

use jj_lib::ref_name::WorkspaceName;

use super::super::Repo;
use super::super::support::load_workspace_internal;
use super::super::workspace_path::is_valid_workspace_name;
use super::listing::existing_dir;
use crate::types::*;

const WORKSPACE_COMMAND: &str = "workspace";

impl Repo {
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

    /// `expected_root` prevents a stale workspace row from forgetting a replacement with the same name.
    pub fn workspace_forget(&self, name: &str, expected_root: Option<&str>) -> CoreResult<()> {
        self.ensure_workspace_is_not_current(name)?;
        if let Some(expected_root) = expected_root {
            self.verify_workspace_root(name, expected_root)?;
        }
        self.forget_workspace_name(name)
    }

    pub(super) fn ensure_workspace_is_not_current(&self, name: &str) -> CoreResult<()> {
        if name == self.workspace_name.as_str() {
            return Err(CoreError::Internal {
                message: "cannot forget the current workspace".to_owned(),
            });
        }
        Ok(())
    }

    pub(super) fn forget_workspace_name(&self, name: &str) -> CoreResult<()> {
        // `--` so an option-shaped workspace name is read as an operand, never as a jj flag.
        self.run_jj_reload(&[WORKSPACE_COMMAND, "forget", "--", name])
    }

    pub(super) fn verify_workspace_root(
        &self,
        name: &str,
        expected_root: &str,
    ) -> CoreResult<PathBuf> {
        let mismatch = |why: &str| CoreError::Internal {
            message: format!("workspace {name} at {expected_root} {why}; refresh and try again"),
        };
        let expected =
            existing_dir(Path::new(expected_root)).ok_or_else(|| mismatch("is not a directory"))?;
        if let Some(recorded) = self.recorded_workspace_root(WorkspaceName::new(name))
            && existing_dir(&recorded).as_ref() != Some(&expected)
        {
            return Err(mismatch("moved"));
        }
        self.verify_workspace_checkout(name, &expected)
            .map_err(|error| mismatch(&format!("is not a jj workspace: {error}")))?;
        Ok(expected)
    }

    pub(super) fn verify_workspace_checkout(&self, name: &str, root: &Path) -> CoreResult<()> {
        let target = load_workspace_internal(root, "verify workspace root")?;
        let same_repo =
            dunce::canonicalize(target.repo_path()).ok().as_ref() == Some(&self.repo_path);
        if target.workspace_name().as_str() != name || !same_repo {
            return Err(CoreError::internal(format!(
                "workspace {name} at {} no longer belongs to this repository",
                root.display()
            )));
        }
        Ok(())
    }
}
