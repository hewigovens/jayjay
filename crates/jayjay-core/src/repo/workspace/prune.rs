use std::path::Path;

use jj_lib::object_id::ObjectId as _;
use jj_lib::ref_name::WorkspaceName;

use super::super::Repo;
use crate::types::*;

pub(crate) enum WorkspacePrune {
    Removed {
        path: String,
        warning: Option<String>,
    },
    Pending {
        changes: usize,
    },
    Stale,
}

impl Repo {
    pub(crate) fn workspace_prune(&self, name: &str) -> JayResult<WorkspacePrune> {
        let workspace = self
            .workspace_list()?
            .into_iter()
            .find(|workspace| workspace.name == name)
            .ok_or_else(|| JayError::internal(format!("no workspace named '{name}'")))?;
        if workspace.is_current {
            return Err(JayError::internal("cannot prune the current workspace"));
        }
        if !workspace.is_path_resolved {
            return Err(JayError::internal(format!(
                "workspace {name} has no directory at '{}'; run `jj workspace forget {name}`",
                workspace.path
            )));
        }
        self.ensure_workspace_does_not_hold_store(name, Path::new(&workspace.path))?;
        match Repo::open(Path::new(&workspace.path))?.refresh_working_copy() {
            Err(JayError::WorkingCopyStale) => return Ok(WorkspacePrune::Stale),
            result => result?,
        }
        self.reload()?;
        let repo = self.get_repo();
        let head = repo
            .view()
            .get_wc_commit_id(WorkspaceName::new(name))
            .ok_or_else(|| JayError::internal(format!("workspace {name} was forgotten meanwhile")))?
            .hex();
        let changes = self
            .revset_commits(
                &repo,
                &format!("(::{head} ~ ::trunk()) ~ (empty() & description(exact:\"\"))"),
            )?
            .len();
        if changes > 0 {
            return Ok(WorkspacePrune::Pending { changes });
        }
        let warning = self.workspace_forget_and_delete(name, &workspace.path)?;
        Ok(WorkspacePrune::Removed {
            path: workspace.path,
            warning,
        })
    }
}
