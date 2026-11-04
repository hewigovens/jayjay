use std::process::Output;

use gix::remote::Direction;
use jj_lib::backend::CommitId;
use jj_lib::git_backend::GitBackend;
use jj_lib::repo::Repo as _;

use crate::repo::Repo;
use crate::types::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PatchSet {
    pub(super) number: u32,
    pub(super) commit: String,
}

impl Repo {
    /// The push URL, since a Gerrit remote may fetch from a mirror.
    pub(super) fn gerrit_push_url(&self) -> JayResult<String> {
        let name = self
            .get_repo()
            .settings()
            .get_string("gerrit.default-remote")
            .unwrap_or_else(|_| self.default_remote());
        let git_repo = self.git_repo("read Gerrit remote")?;
        let missing = || JayError::internal(format!("No remote '{name}' configured"));
        let remote = git_repo.find_remote(name.as_str()).map_err(|_| missing())?;
        let url = remote.url(Direction::Push).ok_or_else(missing)?;
        Ok(url.to_bstring().to_string())
    }

    /// The store jj uses is not the workspace's `.git` in a non-colocated repo.
    fn store_git(&self, args: &[&str], context: &str) -> JayResult<Output> {
        let git_dir = self.git_repo(context)?.git_dir().to_owned();
        let git = self
            .get_repo()
            .settings()
            .get_string("git.executable-path")
            .unwrap_or_else(|_| "git".to_owned());
        let git_dir = format!("--git-dir={}", git_dir.display());
        let mut command_args = vec![git_dir.as_str()];
        command_args.extend_from_slice(args);
        let output = self.registered_output(&git, &command_args, context)?;
        self.ensure_success(&output, context)?;
        Ok(output)
    }

    pub(super) fn latest_patch_set(&self, number: u64) -> JayResult<Option<PatchSet>> {
        let remote = self.gerrit_push_url()?;
        let pattern = format!("{}*", change_ref_prefix(number));
        let output = self.store_git(
            &["ls-remote", "--", &remote, &pattern],
            "list Gerrit patch sets",
        )?;
        Ok(latest_in(&Self::stdout_text(&output), number))
    }

    /// Imported so jj can load the commit and gc keeps it.
    pub(super) fn fetch_patch_set(&self, number: u64, patch_set: &PatchSet) -> JayResult<CommitId> {
        let remote = self.gerrit_push_url()?;
        let reference = format!("{}{}", change_ref_prefix(number), patch_set.number);
        self.store_git(
            &["fetch", "--no-tags", "--", &remote, &reference],
            "fetch Gerrit patch set",
        )?;
        let id = CommitId::try_from_hex(&patch_set.commit)
            .ok_or_else(|| JayError::internal("Gerrit listed an invalid commit"))?;
        let repo = self.get_repo();
        let backend = repo
            .store()
            .backend_impl::<GitBackend>()
            .ok_or_else(|| JayError::internal("Gerrit needs a Git-backed repository"))?;
        backend
            .import_head_commits([&id])
            .map_err(|error| JayError::internal(format!("import Gerrit patch set: {error}")))?;
        Ok(id)
    }
}

fn change_ref_prefix(number: u64) -> String {
    format!("refs/changes/{:02}/{number}/", number % 100)
}

fn latest_in(ls_remote: &str, number: u64) -> Option<PatchSet> {
    let prefix = change_ref_prefix(number);
    ls_remote
        .lines()
        .filter_map(|line| {
            let (commit, reference) = line.split_once('\t')?;
            let number = reference.strip_prefix(&prefix)?.parse().ok()?;
            Some(PatchSet {
                number,
                commit: commit.trim().to_owned(),
            })
        })
        .max_by_key(|patch_set| patch_set.number)
}

#[cfg(test)]
mod tests {
    use super::{PatchSet, latest_in};

    #[test]
    fn the_latest_patch_set_skips_meta_and_other_changes() {
        let ls_remote = "aaa\trefs/changes/01/101/1\n\
            ccc\trefs/changes/01/101/10\n\
            bbb\trefs/changes/01/101/2\n\
            ddd\trefs/changes/01/101/meta\n\
            eee\trefs/changes/01/1101/11\n";
        assert_eq!(
            latest_in(ls_remote, 101),
            Some(PatchSet {
                number: 10,
                commit: "ccc".to_owned()
            })
        );
        assert_eq!(latest_in("", 101), None);
    }
}
