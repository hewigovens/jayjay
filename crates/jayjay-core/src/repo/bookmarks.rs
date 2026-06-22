use std::collections::{BTreeMap, BTreeSet, HashSet};

use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::RefTarget;
use jj_lib::ref_name::RefName;
use jj_lib::repo::Repo as _;

use super::Repo;
use crate::types::*;

impl Repo {
    pub fn list_bookmarks(&self) -> CoreResult<Vec<BookmarkInfo>> {
        let repo = self.get_repo();
        let mut bookmarks = Vec::new();
        let mut local_names: HashSet<String> = HashSet::new();
        for (name, target) in repo.view().local_bookmarks() {
            local_names.insert(name.as_str().to_owned());
            let remote_refs: Vec<_> = repo
                .view()
                .all_remote_bookmarks()
                .filter(|(sym, _)| sym.name == name)
                .collect();
            let tracked_remotes: Vec<String> = remote_refs
                .iter()
                .filter(|(_, remote_ref)| remote_ref.is_tracked())
                .map(|(sym, _)| sym.remote.as_str().to_owned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let available_remotes: Vec<String> = remote_refs
                .iter()
                .map(|(sym, _)| sym.remote.as_str().to_owned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            let (change_id, description) = self.summary_at_target(target);
            bookmarks.push(BookmarkInfo {
                name: name.as_str().to_owned(),
                change_id,
                description,
                is_tracking_remote: !tracked_remotes.is_empty(),
                is_deleted: target.is_absent(),
                is_conflicted: target.has_conflict(),
                tracked_remotes,
                available_remotes,
                has_local_target: true,
            });
        }

        // Synthesize entries for remote bookmarks whose name has no local target.
        let mut orphans: BTreeMap<String, Vec<(String, RefTarget)>> = BTreeMap::new();
        for (sym, remote_ref) in repo.view().all_remote_bookmarks() {
            let name = sym.name.as_str();
            if local_names.contains(name) || remote_ref.target.is_absent() {
                continue;
            }
            orphans
                .entry(name.to_owned())
                .or_default()
                .push((sym.remote.as_str().to_owned(), remote_ref.target.clone()));
        }
        for (name, mut refs) in orphans {
            refs.sort_by(|a, b| a.0.cmp(&b.0));
            let remotes: Vec<String> = refs.iter().map(|(r, _)| r.clone()).collect();
            let first_target = &refs[0].1;
            let (change_id, description) = self.summary_at_target(first_target);
            bookmarks.push(BookmarkInfo {
                name,
                change_id,
                description,
                is_tracking_remote: false,
                is_deleted: false,
                is_conflicted: first_target.has_conflict(),
                tracked_remotes: Vec::new(),
                available_remotes: remotes,
                has_local_target: false,
            });
        }

        Ok(bookmarks)
    }

    fn summary_at_target(&self, target: &RefTarget) -> (String, String) {
        let Some(commit_id) = target.as_normal() else {
            return (String::new(), String::new());
        };
        match self.get_repo().store().get_commit(commit_id) {
            Ok(commit) => (
                encode_reverse_hex(commit.change_id().as_bytes()),
                commit.description().lines().next().unwrap_or("").to_owned(),
            ),
            Err(_) => (String::new(), String::new()),
        }
    }

    pub fn create_bookmark(&self, name: &str, rev: &str) -> CoreResult<()> {
        self.with_resolved_commit_transaction(
            rev,
            "create bookmark",
            false,
            |_, commit, repo_mut| {
                self.set_bookmark_target(
                    repo_mut,
                    name,
                    RefTarget::resolved(Some(commit.id().clone())),
                );
                Ok(())
            },
        )
    }

    pub fn move_bookmark(&self, name: &str, to_rev: &str) -> CoreResult<()> {
        self.with_resolved_commit_transaction(
            to_rev,
            "move bookmark",
            false,
            |_, commit, repo_mut| {
                self.set_bookmark_target(
                    repo_mut,
                    name,
                    RefTarget::resolved(Some(commit.id().clone())),
                );
                Ok(())
            },
        )
    }

    pub fn delete_bookmark(&self, name: &str) -> CoreResult<()> {
        self.update_local_bookmark(name, RefTarget::absent(), "delete bookmark")
    }

    pub fn rename_bookmark(&self, old_name: &str, new_name: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let target = repo
            .view()
            .get_local_bookmark(RefName::new(old_name))
            .clone();
        if target.is_absent() {
            return Err(CoreError::Internal {
                message: format!("bookmark '{old_name}' not found"),
            });
        }
        self.with_repo_transaction("rename bookmark", false, move |_, repo_mut| {
            self.set_bookmark_target(repo_mut, new_name, target);
            self.set_bookmark_target(repo_mut, old_name, RefTarget::absent());
            Ok(())
        })
    }

    pub fn track_bookmark(&self, name: &str, remote: &str) -> CoreResult<()> {
        self.run_jj_reload(&[
            "bookmark",
            "track",
            &format!("--remote={remote}"),
            "--",
            name,
        ])
    }

    /// Prune stale remote refs, then forget bookmarks that are locally deleted
    /// or have no remote counterpart.
    /// Uses `jj bookmark forget` so it won't propagate deletions to remotes.
    /// Returns the number of bookmarks forgotten.
    pub fn forget_stale_bookmarks(&self) -> CoreResult<u32> {
        // Step 1: Prune remote tracking refs via git fetch
        let _ = self.run_jj(&["git", "fetch", "--remote", "origin"]);

        // Step 2: Delete local git branches whose remote is gone
        // (equivalent to: git branch -vv | grep ': gone]' | awk '{print $1}' | xargs git branch -D)
        let output = self.command_output("git", &["branch", "-vv"], "list git branches")?;
        let gone_branches: Vec<String> = Self::stdout_text(&output)
            .lines()
            .filter(|line| line.contains(": gone]"))
            .filter_map(|line| line.split_whitespace().next())
            .map(|s| s.to_owned())
            .collect();
        for branch in &gone_branches {
            let _ =
                self.command_output("git", &["branch", "-D", "--", branch], "delete gone branch");
        }

        // Step 3: Re-import git refs so jj sees the deletions
        if !gone_branches.is_empty() {
            let _ = self.run_jj(&["git", "import"]);
        }
        self.reload()?;

        // Step 4: Forget jj bookmarks that are locally deleted or have no remote
        let bookmarks = self.list_bookmarks()?;
        let stale: Vec<&str> = bookmarks
            .iter()
            .filter(|b| b.is_deleted || b.available_remotes.is_empty())
            .map(|b| b.name.as_str())
            .collect();
        let count = gone_branches.len() as u32 + stale.len() as u32;
        for name in stale {
            self.run_jj(&["bookmark", "forget", "--", name])?;
        }
        if count > 0 {
            self.reload()?;
        }
        Ok(count)
    }
}
