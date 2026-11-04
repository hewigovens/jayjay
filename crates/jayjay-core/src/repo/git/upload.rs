use std::collections::BTreeSet;

use jj_lib::backend::CommitId;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;

use super::remote::remote_url_is_gerrit;
use super::sync::configured_command;
use crate::repo::{Repo, SyncToken};
use crate::types::*;

const UPLOAD_COMMAND: &str = "jayjay.upload-command";
const REVISION_PLACEHOLDER: &str = "$revision";
const GERRIT_SETTINGS: [&str; 3] = [
    "gerrit.default-remote",
    "gerrit.default-remote-branch",
    "gerrit.review-url",
];

enum UploadCommand {
    Gerrit,
    Custom(Vec<String>),
}

impl Repo {
    fn upload_command(&self) -> Option<UploadCommand> {
        let repo = self.get_repo();
        let settings = repo.settings();
        if let Some(args) = configured_command(settings, UPLOAD_COMMAND) {
            return Some(UploadCommand::Custom(args));
        }
        let gerrit = GERRIT_SETTINGS
            .iter()
            .any(|key| settings.get_string(*key).is_ok())
            || self
                .git_remote_url()
                .is_ok_and(|url| remote_url_is_gerrit(&url));
        gerrit.then_some(UploadCommand::Gerrit)
    }

    pub fn upload_title(&self) -> Option<String> {
        Some(match self.upload_command()? {
            UploadCommand::Gerrit => jayjay_i18n::repo::upload_to_gerrit(),
            UploadCommand::Custom(_) => jayjay_i18n::repo::upload_change(),
        })
    }

    pub fn upload(&self, revision: &str, sync: &SyncToken) -> JayResult<String> {
        let _enter = sync.enter();
        let command = self
            .upload_command()
            .ok_or_else(|| JayError::internal("This repository has no upload command"))?;
        let commit = self.snapshot_and_follow_one(revision)?;
        let args = match &command {
            UploadCommand::Gerrit => self.gerrit_upload_args(commit),
            UploadCommand::Custom(args) => args
                .iter()
                .map(|arg| arg.replace(REVISION_PLACEHOLDER, &commit))
                .collect(),
        };
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self.run_remote_command(&args, "upload failed", sync)?;
        Ok(match command {
            UploadCommand::Gerrit => gerrit_summary(output),
            UploadCommand::Custom(_) => output,
        })
    }

    fn gerrit_upload_args(&self, commit: String) -> Vec<String> {
        let mut args = vec![
            "gerrit".to_owned(),
            "upload".to_owned(),
            "-r".to_owned(),
            commit,
        ];
        let configured = self
            .get_repo()
            .settings()
            .get_string("gerrit.default-remote-branch")
            .is_ok();
        if !configured && let Some(branch) = self.trunk_remote_branch() {
            args.push(format!("--remote-branch={branch}"));
        }
        args
    }

    /// Where Gerrit changes land unless `gerrit.default-remote-branch` says otherwise.
    fn trunk_remote_branch(&self) -> Option<String> {
        let repo = self.get_repo();
        let trunk: Vec<CommitId> = self
            .revset_commit_ids(&repo, "trunk()")
            .iter()
            .filter_map(CommitId::try_from_hex)
            .collect();
        let names: BTreeSet<&str> = repo
            .view()
            .all_remote_bookmarks()
            .filter(|(symbol, remote)| {
                symbol.remote != REMOTE_NAME_FOR_LOCAL_GIT_REPO
                    && remote.target.present_adds().any(|id| trunk.contains(id))
            })
            .map(|(symbol, _)| symbol.name.as_str())
            .collect();
        match names.into_iter().collect::<Vec<_>>().as_slice() {
            [name] => Some((*name).to_owned()),
            _ => None,
        }
    }
}

/// Keeps Gerrit's `remote:   <url> <subject>` lines and drops push progress.
fn gerrit_summary(output: String) -> String {
    let changes: Vec<&str> = output
        .lines()
        .filter_map(|line| line.strip_prefix("remote:"))
        .map(str::trim)
        .filter(|line| line.starts_with("https://") || line.starts_with("http://"))
        .collect();
    if changes.is_empty() {
        output
    } else {
        changes.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::gerrit_summary;

    #[test]
    fn gerrit_summary_keeps_only_the_change_links() {
        let output = "Found 2 heads to push to Gerrit (remote 'origin'), target branch 'main'.\n\
            Pushing vpwrvwuu 512c274e Add hello\n\
            remote: remote: Processing changes: refs: 2, new: 1\r        remote: Processing changes: refs: 2, new: 1, done        \n\
            remote: \n\
            remote: SUCCESS        \n\
            remote: \n\
            remote:   https://review.example.org/c/demo/+/1 Add hello [NEW]        \n\
            remote:   https://review.example.org/c/demo/+/2 Add world        \n\
            remote:";

        assert_eq!(
            gerrit_summary(output.to_owned()),
            "https://review.example.org/c/demo/+/1 Add hello [NEW]\nhttps://review.example.org/c/demo/+/2 Add world"
        );
        assert_eq!(
            gerrit_summary("Nothing changed.".to_owned()),
            "Nothing changed."
        );
    }
}
