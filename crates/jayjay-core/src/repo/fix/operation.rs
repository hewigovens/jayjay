use jj_lib::fileset::FilesetParseContext;
use jj_lib::fix::{FileToFix, ParallelFileFixer, fix_files};
use jj_lib::matchers::EverythingMatcher;
use jj_lib::settings::UserSettings;
use jj_lib::store::Store;

use super::runner::FixRunner;
use super::tools::{FixTool, parse_fix_tools};
use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    pub fn fix_unavailable_reason(&self) -> Option<String> {
        self.fix_tools(self.get_repo().settings())
            .err()
            .map(|error| error.to_string())
    }

    /// `jj fix -s`: each change in `revs` and its descendants.
    pub fn fix(&self, revs: &[String]) -> JayResult<FixSummary> {
        if revs.is_empty() {
            return Err(JayError::internal("fix requires at least one change"));
        }
        let _write = self.write_guard()?;
        let (repo, commits) = self.snapshot_and_follow_commits(revs)?;
        for (commit, rev) in commits.iter().zip(revs) {
            self.ensure_commit_mutable(&repo, commit, rev)?;
        }
        let runner = FixRunner::new(self.fix_tools(repo.settings())?, self.path.clone());
        let mut fixer =
            ParallelFileFixer::new(|store: &Store, file: &FileToFix| runner.fix_file(store, file));
        let mut tx = repo.start_transaction();
        let ids = commits.iter().map(|commit| commit.id().clone()).collect();
        let summary = block_on_result(
            "fix changes",
            fix_files(ids, &EverythingMatcher, false, tx.repo_mut(), &mut fixer),
        )?;
        if tx.repo().has_changes() {
            self.commit_transaction_rebase(
                tx,
                &format!("fixed {} changes", summary.num_fixed_commits),
            )?;
        }
        Ok(FixSummary {
            checked_changes: summary.num_checked_commits as u32,
            fixed_changes: summary.num_fixed_commits as u32,
            failures: runner.take_failures(),
        })
    }

    fn fix_tools(&self, settings: &UserSettings) -> JayResult<Vec<FixTool>> {
        let aliases_map = self.fileset_aliases_map(settings)?;
        let path_converter = self.path_converter();
        let context = FilesetParseContext {
            aliases_map: &aliases_map,
            path_converter: &path_converter,
        };
        let tools = parse_fix_tools(settings, &context)?;
        if tools.is_empty() {
            return Err(JayError::internal(
                "No [fix.tools] formatter is enabled. Add one to your jj config to run formatters.",
            ));
        }
        Ok(tools)
    }
}
