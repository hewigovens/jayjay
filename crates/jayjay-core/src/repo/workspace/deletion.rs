use std::fs;
use std::path::PathBuf;

use super::super::Repo;
use crate::filesystem::io_error;
use crate::types::*;

impl Repo {
    /// Moves the verified checkout aside before forgetting, so later path reuse cannot redirect deletion.
    pub fn workspace_forget_and_delete(
        &self,
        name: &str,
        expected_root: &str,
    ) -> CoreResult<Option<String>> {
        self.ensure_workspace_is_not_current(name)?;
        let staged = self.stage_workspace_for_deletion(name, expected_root)?;
        if let Err(error) = self.forget_workspace_name(name) {
            return Err(staged.restore_or_recovery_error(error));
        }

        Ok(staged.delete(self, name).err().map(|error| {
            format!(
                "Workspace {name} was forgotten, but its directory could not be deleted. It remains at:\n{}\n{error}",
                staged.root.display()
            )
        }))
    }

    fn stage_workspace_for_deletion(
        &self,
        name: &str,
        expected_root: &str,
    ) -> CoreResult<StagedWorkspace> {
        let original_root = self.verify_workspace_root(name, expected_root)?;
        let parent = original_root.parent().ok_or_else(|| {
            CoreError::internal(format!(
                "workspace {name} at {} has no parent directory",
                original_root.display()
            ))
        })?;
        let root = tempfile::Builder::new()
            .prefix(".jayjay-delete-")
            .tempdir_in(parent)
            .map_err(|error| io_error("stage workspace deletion beside", &original_root, error))?
            .keep();
        fs::remove_dir(&root)
            .map_err(|error| io_error("prepare workspace staging path", &root, error))?;
        if let Err(error) = fs::rename(&original_root, &root) {
            return Err(io_error(
                "stage workspace deletion for",
                &original_root,
                error,
            ));
        }
        let staged = StagedWorkspace {
            original_root,
            root,
        };
        if let Err(error) = self.verify_workspace_checkout(name, &staged.root) {
            return Err(staged.restore_or_recovery_error(error));
        }
        Ok(staged)
    }
}

struct StagedWorkspace {
    original_root: PathBuf,
    root: PathBuf,
}

impl StagedWorkspace {
    fn restore_or_recovery_error(&self, error: CoreError) -> CoreError {
        match self.restore() {
            Ok(()) => error,
            Err(restore_error) => CoreError::internal(format!(
                "{error}; restoring the checkout also failed: {restore_error}. The checkout remains at {}",
                self.root.display()
            )),
        }
    }

    fn restore(&self) -> CoreResult<()> {
        if self.original_root.exists() {
            return Err(CoreError::internal(format!(
                "the original path is occupied: {}",
                self.original_root.display()
            )));
        }
        fs::rename(&self.root, &self.original_root)
            .map_err(|error| io_error("restore workspace to", &self.original_root, error))
    }

    fn delete(&self, repo: &Repo, name: &str) -> CoreResult<()> {
        repo.verify_workspace_checkout(name, &self.root)?;
        fs::remove_dir_all(&self.root)
            .map_err(|error| io_error("delete workspace", &self.root, error))
    }
}

#[cfg(test)]
mod tests {
    use jj_test::init_jj_repo;

    use super::Repo;

    #[test]
    fn staged_workspace_deletion_keeps_a_replacement_at_the_original_path() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");
        let dest = temp_dir.path().join("feature-ws");
        repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
            .expect("add workspace");
        let staged = repo
            .stage_workspace_for_deletion("feature", dest.to_str().expect("utf8 dest"))
            .expect("stage workspace");

        std::fs::create_dir(&dest).expect("reuse original path");
        let marker = dest.join("unrelated");
        std::fs::write(&marker, "keep me").expect("write replacement marker");
        staged
            .delete(&repo, "feature")
            .expect("delete staged checkout");

        assert_eq!(
            std::fs::read_to_string(&marker).expect("replacement survived"),
            "keep me"
        );
        repo.workspace_forget("feature", None)
            .expect("clean up workspace metadata");
    }
}
