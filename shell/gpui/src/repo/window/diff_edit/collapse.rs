use gpui::{AppContext, Context};
use jayjay_core::{
    CoreResult, FileDiffStats, diff_edit_auto_collapsed_paths,
    diff_edit_collapses_while_stats_pending, diff_edit_starts_collapsed,
};

use crate::repo::window::RepoWindow;

impl RepoWindow {
    pub(super) fn selected_commit_id(&self, cx: &Context<Self>) -> Option<String> {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .map(|change| change.commit_id.id.clone())
    }

    /// Seeds collapse from the already-loaded whole-change stats so a large diff never renders expanded while per-file stats compute; the per-file pass replaces this approximation with the precise policy.
    pub(super) fn seed_diff_edit_collapse(&mut self, cx: &Context<Self>) {
        let vm = self.vm.read(cx);
        let Some(files) = vm.files.as_ref() else {
            return;
        };
        let collapse_all = match vm.change_stats.as_ref() {
            Some(stats) => {
                let total = u64::from(stats.insertions) + u64::from(stats.deletions);
                diff_edit_starts_collapsed(files.len(), total)
            }
            None => diff_edit_collapses_while_stats_pending(files.len()),
        };
        if collapse_all {
            self.diff_edit.collapsed = files.iter().map(|hunk| hunk.path.clone()).collect();
        }
    }

    pub fn toggle_diff_edit_collapse(&mut self, path: &str, cx: &mut Context<Self>) {
        if !self.diff_edit.collapsed.remove(path) {
            self.diff_edit.collapsed.insert(path.to_owned());
        }
        self.diff_edit.collapse_touched = true;
        self.diff_edit.rows = None;
        cx.notify();
    }

    pub fn collapse_all_diff_edit(&mut self, cx: &mut Context<Self>) {
        let Some(files) = self.vm.read(cx).files.clone() else {
            return;
        };
        self.diff_edit.collapsed = files.iter().map(|hunk| hunk.path.clone()).collect();
        self.diff_edit.collapse_touched = true;
        self.diff_edit.rows = None;
        cx.notify();
    }

    pub fn expand_all_diff_edit(&mut self, cx: &mut Context<Self>) {
        self.diff_edit.collapsed.clear();
        self.diff_edit.collapse_touched = true;
        self.diff_edit.rows = None;
        cx.notify();
    }

    pub fn diff_edit_collapsed(&self, path: &str) -> bool {
        self.diff_edit.collapsed.contains(path)
    }

    pub fn diff_edit_stats_ready(&self) -> bool {
        self.diff_edit.stats.is_some()
    }

    pub(super) fn spawn_diff_edit_stats(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            return;
        };
        // The immutable commit id is the query revision: a change-id query could read an amended replacement that the completion guard still attributes to the on-screen commit.
        let Some(commit) = self
            .vm
            .read(cx)
            .selected_change_for_file_ops()
            .map(|change| change.commit_id.id.clone())
        else {
            return;
        };
        self.diff_edit.stats_commit = Some(commit.clone());
        let session = self.diff_edit.session;
        let ignore_whitespace = self.vm.read(cx).ignore_whitespace;
        cx.spawn(async move |this, cx| {
            let rev = commit.clone();
            let stats = cx
                .background_spawn(async move { repo.diff_file_stats(&rev, ignore_whitespace) })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.finish_diff_edit_stats(session, commit, ignore_whitespace, stats, cx)
            });
        })
        .detach();
    }

    fn finish_diff_edit_stats(
        &mut self,
        session: u64,
        commit: String,
        ignore_whitespace: bool,
        stats: CoreResult<Vec<FileDiffStats>>,
        cx: &mut Context<Self>,
    ) {
        if !self.diff_edit.active || self.diff_edit.session != session {
            return;
        }
        // Same session and commit both survive a mode toggle and an amend keeps the change id; the commit id and captured mode together prove the stats describe what is on screen.
        if self.selected_commit_id(cx).as_deref() != Some(commit.as_str())
            || self.vm.read(cx).ignore_whitespace != ignore_whitespace
        {
            return;
        }
        let Ok(stats) = stats else {
            return;
        };
        // The aggregate seed can overcount the displayed rows (placeholders, projections), so the precise pass recomputes the whole policy and replaces the seed outright.
        if !self.diff_edit.collapse_touched {
            let cards: Vec<String> = self
                .vm
                .read(cx)
                .files
                .as_ref()
                .map(|files| files.iter().map(|hunk| hunk.path.clone()).collect())
                .unwrap_or_else(|| stats.iter().map(|s| s.path.clone()).collect());
            let total: u64 = stats
                .iter()
                .map(|s| u64::from(s.insertions) + u64::from(s.deletions))
                .sum();
            self.diff_edit.collapsed = if diff_edit_starts_collapsed(cards.len(), total) {
                cards.into_iter().collect()
            } else {
                diff_edit_auto_collapsed_paths(&stats).into_iter().collect()
            };
            self.diff_edit.rows = None;
        }
        self.diff_edit.stats = Some(stats.into_iter().map(|s| (s.path.clone(), s)).collect());
        cx.notify();
    }
}
