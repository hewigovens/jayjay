use jj_lib::object_id::ObjectId;
use jj_lib::ref_name::{RefName, RemoteName, RemoteRefSymbol, WorkspaceName};

use super::plan::{PrHeadRepo, PullRequestImportPlan};
use super::{github, plan, url};
use crate::repo::Repo;
use crate::repo::command_process::SyncToken;
use crate::repo::git::remote_url_uses_ssh;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::repo::support::{canonicalize, unique_name};
use crate::repo::workspace_path::is_valid_workspace_name;
use crate::types::*;

const UNSUPPORTED_URL: &str = "Not a supported pull request URL. Paste a GitHub /pull/ URL; GitLab and Codeberg are coming soon.";

impl Repo {
    pub fn pull_request_import_preview(
        &self,
        url: &str,
        sync: &SyncToken,
    ) -> CoreResult<PullRequestImportPreview> {
        let _enter = sync.enter();
        let plan = self.resolve_pull_request_import(url)?;
        let (name, dest) = self.suggest_workspace_destination(plan.resolved.number)?;
        let existing_workspace = self.existing_pull_request_workspace(&plan)?;
        Ok(plan.into_preview(name, dest, existing_workspace))
    }

    pub fn pull_request_import(
        &self,
        url: &str,
        previewed_head_commit_id: &str,
        workspace_name: &str,
        workspace_dest: &str,
        sync: &SyncToken,
    ) -> CoreResult<String> {
        let plan = {
            let _enter = sync.enter();
            self.resolve_pull_request_import(url)?
        };
        if !plan.resolved.has_head(previewed_head_commit_id) {
            return Err(CoreError::internal(format!(
                "The pull request head moved to {} since the preview; the preview now shows the new head.",
                plan.resolved.short_head(),
            )));
        }
        self.import_resolved(&plan, workspace_name, workspace_dest, sync)
    }

    pub(super) fn import_resolved(
        &self,
        plan: &PullRequestImportPlan,
        workspace_name: &str,
        workspace_dest: &str,
        sync: &SyncToken,
    ) -> CoreResult<String> {
        // Relative locations resolve against the repository's parent, where the preview suggests them.
        let workspace_dest = self.workspace_parent().join(workspace_dest);
        let workspace_dest = workspace_dest.to_string_lossy();
        if let Some(conflict) = self.workspace_conflict(workspace_name, &workspace_dest) {
            return Err(CoreError::internal(conflict));
        }
        self.fetch_pull_request_head(plan, sync)?;
        // Outside the token: cancelling a half-created workspace would leave a registered name with a partial directory.
        self.workspace_add(
            &workspace_dest,
            workspace_name,
            &plan.resolved.head_commit_id,
        )?;
        Ok(workspace_dest.into_owned())
    }

    fn fetch_pull_request_head(
        &self,
        plan: &PullRequestImportPlan,
        sync: &SyncToken,
    ) -> CoreResult<()> {
        let _enter = sync.enter();
        sync.check()?;

        let (remote, branch) = (plan.remote.name(), plan.resolved.head_branch.as_str());
        let added_remote = !plan.remote.exists();
        if added_remote {
            self.git_remote_add(remote, plan.remote.url())?;
        }
        sync.check()?;

        self.git_fetch_raw(remote, branch)
            .map_err(|error| match error {
                CoreError::Canceled => error,
                _ if added_remote => CoreError::internal(format!(
                    "Remote '{remote}' was added, but fetching the PR head failed: {error}"
                )),
                _ => error,
            })?;
        sync.check()?;

        // Fetching a deleted or force-pushed branch still succeeds, and the head may already be local from an earlier import, so check the fetched bookmark itself.
        if !self.fetched_branch_has_head(plan) {
            return Err(CoreError::internal(format!(
                "'{branch}' on remote '{remote}' no longer contains the pull request head {}; the branch was deleted or force-pushed.",
                plan.resolved.short_head(),
            )));
        }
        sync.check()
    }

    fn workspace_parent(&self) -> std::path::PathBuf {
        let path = canonicalize(&self.path);
        path.parent().unwrap_or(&path).to_path_buf()
    }

    fn resolve_pull_request_import(&self, url: &str) -> CoreResult<PullRequestImportPlan> {
        let parsed =
            url::parse_pull_request_url(url).ok_or_else(|| CoreError::internal(UNSUPPORTED_URL))?;
        let origin_url = self.git_remote_url()?;
        let origin = HostedRepo::parse(&origin_url).ok_or_else(|| {
            CoreError::internal("This repository's origin is not on a supported host.")
        })?;
        if !origin.is_same_repository(&parsed.base) {
            return Err(CoreError::internal(format!(
                "This pull request targets {}, but this repository's origin is {}.",
                parsed.base.slug(),
                origin.slug(),
            )));
        }
        let resolved = match parsed.base.host {
            RepoHost::GitHub => github::resolve(self, &parsed)?.checked(parsed.number),
            RepoHost::GitLab | RepoHost::Codeberg | RepoHost::Cursor => None,
        }
        .ok_or_else(|| {
            CoreError::internal(format!(
                "Couldn't resolve this pull request on {}; only GitHub is supported so far.",
                parsed.base.host.display_name(),
            ))
        })?;

        let remotes = self.git_remotes()?;
        let remote = match &resolved.head {
            PrHeadRepo::SameRepository => plan::RemoteChoice::Reuse {
                name: String::from("origin"),
                url: origin_url.clone(),
            },
            PrHeadRepo::Fork(fork) => plan::choose_remote(
                &remotes,
                &fork.owner,
                &fork.clone_url(remote_url_uses_ssh(&origin_url)),
            ),
        };

        Ok(PullRequestImportPlan {
            base: origin,
            resolved,
            remote,
        })
    }

    fn fetched_branch_has_head(&self, plan: &PullRequestImportPlan) -> bool {
        let repo = self.get_repo();
        let fetched = repo.view().get_remote_bookmark(RemoteRefSymbol {
            name: RefName::new(&plan.resolved.head_branch),
            remote: RemoteName::new(plan.remote.name()),
        });
        let tips = fetched
            .target
            .added_ids()
            .map(|id| id.hex())
            .collect::<Vec<_>>()
            .join(" | ");
        let head = &plan.resolved.head_commit_id;
        !tips.is_empty()
            && !self
                .revset_commit_ids(&repo, &format!("present({head}) & ::({tips})"))
                .is_empty()
    }

    /// The last-fetched head bookmark counts too, so a PR that gained commits still finds its workspace; a head already in trunk matches nothing.
    pub(super) fn existing_pull_request_workspace(
        &self,
        plan: &PullRequestImportPlan,
    ) -> CoreResult<Option<PullRequestImportWorkspace>> {
        let repo = self.get_repo();
        let fetched = repo.view().get_remote_bookmark(RemoteRefSymbol {
            name: RefName::new(&plan.resolved.head_branch),
            remote: RemoteName::new(plan.remote.name()),
        });
        let heads = std::iter::once(plan.resolved.head_commit_id.clone())
            .chain(fetched.target.added_ids().map(|id| id.hex()))
            .map(|id| format!("present({id})"))
            .collect::<Vec<_>>()
            .join(" | ");
        let working_copies = self.revset_commit_ids(
            &repo,
            &format!("working_copies() & (({heads}) ~ ::trunk())::"),
        );
        let names: Vec<String> = repo
            .view()
            .wc_commit_ids()
            .iter()
            .filter(|(_, id)| working_copies.contains(&id.hex()))
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        if names.is_empty() {
            return Ok(None);
        }
        Ok(self
            .workspace_list()?
            .into_iter()
            .find(|workspace| names.contains(&workspace.name) && workspace.is_path_resolved)
            .map(|workspace| PullRequestImportWorkspace {
                name: workspace.name,
                dest: workspace.path,
            }))
    }

    /// `jj workspace add` rejects these too, but only after the remote was added and fetched.
    fn workspace_conflict(&self, name: &str, dest: &str) -> Option<String> {
        let repo = self.get_repo();
        let occupied = std::fs::read_dir(dest).is_ok_and(|mut entries| entries.next().is_some());
        if !is_valid_workspace_name(name) {
            Some(format!("invalid workspace name: {name}"))
        } else if repo
            .view()
            .get_wc_commit_id(WorkspaceName::new(name))
            .is_some()
        {
            Some(format!("A workspace named '{name}' already exists."))
        } else if occupied || std::path::Path::new(dest).is_file() {
            Some(format!(
                "{dest} already exists and is not an empty directory."
            ))
        } else {
            None
        }
    }

    pub(super) fn suggest_workspace_destination(
        &self,
        pr_number: u32,
    ) -> CoreResult<(String, String)> {
        let base = format!("pr-{pr_number}");
        let taken_names: Vec<String> = self
            .workspace_list()?
            .into_iter()
            .map(|workspace| workspace.name)
            .collect();
        let parent = self.workspace_parent();
        let name = unique_name(&base, |candidate| {
            taken_names.iter().any(|name| name == candidate) || parent.join(candidate).exists()
        });
        let dest = parent.join(&name).to_string_lossy().into_owned();
        Ok((name, dest))
    }
}

impl PullRequestImportPlan {
    fn into_preview(
        self,
        name: String,
        dest: String,
        existing_workspace: Option<PullRequestImportWorkspace>,
    ) -> PullRequestImportPreview {
        PullRequestImportPreview {
            pull_request: PullRequestImportSource {
                host: self.base.host.display_name().to_owned(),
                base_repo: self.base.slug(),
                number: self.resolved.number,
                state: self.resolved.state,
                title: self.resolved.title,
                url: self.resolved.url,
            },
            remote: self.remote.into_preview(self.resolved.head_branch),
            head_commit_id: self.resolved.head_commit_id,
            same_repository: matches!(self.resolved.head, PrHeadRepo::SameRepository),
            workspace: PullRequestImportWorkspace { name, dest },
            existing_workspace,
        }
    }
}

impl plan::RemoteChoice {
    fn into_preview(self, bookmark: String) -> PullRequestImportRemote {
        let exists = self.exists();
        let (name, url) = match self {
            plan::RemoteChoice::Reuse { name, url } | plan::RemoteChoice::Add { name, url } => {
                (name, url)
            }
        };
        PullRequestImportRemote {
            name,
            url,
            exists,
            bookmark,
        }
    }
}
