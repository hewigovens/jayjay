use std::collections::HashMap;

use jj_lib::git::{GitImportOptions, GitSettings};
use jj_lib::repo::Repo as _;
use jj_lib::revset::{self, RevsetDiagnostics};
use jj_lib::settings::UserSettings;

use crate::repo::Repo;
use crate::repo::support::{block_on_result, load_workspace_internal};
use crate::types::*;

impl Repo {
    /// `jj git import`: pick up refs Git moved behind jj's back, such as branches a raw `git` command created or deleted. HEAD comes first, before the snapshot, so a commit Git just made is not also recorded as pending edits of the old `@`.
    pub(crate) fn git_import(&self) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.import_git_head()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let options = git_import_options(repo.settings())?;
        let mut tx = repo.start_transaction();
        block_on_result(
            "import git refs",
            jj_lib::git::import_refs(tx.repo_mut(), &options),
        )?;
        self.commit_transaction_rebase(tx, "import git refs")
    }

    /// A Git command that moved HEAD (a raw `git commit`, say) already left the files on the new commit, so start a fresh working-copy change there without touching disk, as the CLI does at the start of every command.
    fn import_git_head(&self) -> CoreResult<()> {
        let context = "import git head";
        let mut workspace = load_workspace_internal(&self.path, context)?;
        let repo_loader = workspace.repo_loader().clone();
        // Lock before loading the head, as a snapshot does, so no operation lands between the load and the working-copy reset.
        let mut locked_ws = block_on_result(context, workspace.start_working_copy_mutation())?;
        let repo = block_on_result(context, repo_loader.load_at_head())?;
        let mut tx = repo.start_transaction();
        if self.is_colocated(repo.store()) {
            block_on_result(
                context,
                jj_lib::git::import_head(tx.repo_mut(), &self.workspace_name, &self.path),
            )?;
        }
        if !tx.repo().has_changes() {
            block_on_result(context, locked_ws.finish(repo.op_id().clone()))?;
            self.set_repo(repo);
            return Ok(());
        }
        if let Some(head_id) = tx
            .repo()
            .view()
            .git_head(&self.workspace_name)
            .as_normal()
            .cloned()
        {
            let head = repo
                .store()
                .get_commit(&head_id)
                .map_err(|error| CoreError::internal(format!("load git head: {error}")))?;
            let wc_commit = block_on_result(
                "check out git head",
                tx.repo_mut().check_out(self.workspace_name.clone(), &head),
            )?;
            block_on_result(context, locked_ws.locked_wc().reset(&wc_commit))?;
        }
        block_on_result("rebase descendants", tx.repo_mut().rebase_descendants())?;
        self.sync_colocated_git(&mut tx)?;
        let new_repo = block_on_result(context, tx.commit(context))?;
        block_on_result(context, locked_ws.finish(new_repo.op_id().clone()))?;
        self.set_repo(new_repo);
        Ok(())
    }
}

/// The CLI's import options: `git.*` settings plus each remote's `auto-track-bookmarks` matcher.
fn git_import_options(settings: &UserSettings) -> CoreResult<GitImportOptions> {
    let git_settings = GitSettings::from_settings(settings).map_err(CoreError::internal)?;
    let mut remote_auto_track_bookmarks = HashMap::new();
    for (name, remote) in settings.remote_settings().map_err(CoreError::internal)? {
        let Some(text) = remote.auto_track_bookmarks else {
            continue;
        };
        let expression = revset::parse_string_expression(&mut RevsetDiagnostics::new(), &text)
            .map_err(|error| {
                CoreError::internal(format!(
                    "invalid remotes.{}.auto-track-bookmarks: {error}",
                    name.as_symbol()
                ))
            })?;
        remote_auto_track_bookmarks.insert(name, expression.to_matcher());
    }
    Ok(GitImportOptions {
        abandon_unreachable_commits: git_settings.abandon_unreachable_commits,
        record_synthetic_predecessors: git_settings.record_synthetic_predecessors,
        remote_auto_track_bookmarks,
    })
}

#[cfg(test)]
mod tests {
    use jj_test::{init_jj_repo, run_git, run_jj_in};

    use crate::repo::Repo;

    #[test]
    fn git_import_picks_up_a_branch_git_created() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");
        let head = repo.log("@").expect("log")[0].commit_id.id.clone();
        run_git(&repo_path, &["branch", "from-git", &head]);

        repo.git_import().expect("import");

        let bookmark = repo
            .list_bookmarks()
            .expect("list bookmarks")
            .into_iter()
            .find(|bookmark| bookmark.name == "from-git")
            .expect("imported bookmark");
        assert!(bookmark.has_local_target);
    }

    #[test]
    fn git_import_follows_a_head_git_moved() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        run_jj_in(&repo_path, &["new"]);
        let repo = Repo::open(&repo_path).expect("open repo");
        std::fs::write(repo_path.join("from-git.txt"), "committed by git\n").expect("write");
        run_git(&repo_path, &["add", "from-git.txt"]);
        run_git(
            &repo_path,
            &[
                "-c",
                "user.name=Git",
                "-c",
                "user.email=git@example.com",
                "commit",
                "-m",
                "from git",
            ],
        );
        let git_head = run_git(&repo_path, &["rev-parse", "HEAD"]).stdout;

        repo.git_import().expect("import");

        let working_copy = &repo.log("@").expect("log")[0];
        assert!(
            working_copy.is_empty,
            "the file git committed must not count as pending edits"
        );
        assert_eq!(
            format!("{}\n", working_copy.parents[0]).into_bytes(),
            git_head,
            "@ must sit on the commit git created"
        );
        assert_eq!(
            repo.log("heads(all())").expect("heads").len(),
            1,
            "the old working copy must not survive as a parallel head"
        );
        assert_eq!(
            run_git(&repo_path, &["rev-parse", "HEAD"]).stdout,
            git_head,
            "importing must not move git HEAD back"
        );
    }
}
