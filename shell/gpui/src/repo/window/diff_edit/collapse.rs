use gpui::{AppContext, Context};
use jayjay_core::{CoreResult, FileDiffStats};

use crate::repo::window::RepoWindow;

impl RepoWindow {
    pub(super) fn selected_commit_id(&self, cx: &Context<Self>) -> Option<String> {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .map(|change| change.commit_id.id.clone())
    }

    pub(super) fn seed_diff_edit_collapse(&mut self, cx: &Context<Self>) {
        let vm = self.vm.read(cx);
        let Some(files) = vm.files.as_ref() else {
            return;
        };
        let paths: Vec<String> = files.iter().map(|hunk| hunk.path.clone()).collect();
        let total = vm
            .change_stats
            .as_ref()
            .map(|stats| u64::from(stats.insertions) + u64::from(stats.deletions));
        self.diff_edit.session.seed_collapse(&paths, total);
    }

    /// Card folds change the row list, so every collapse mutation drops the cached model.
    pub(super) fn invalidate_diff_edit_rows(&mut self, cx: &mut Context<Self>) {
        self.diff_edit.rows = None;
        cx.notify();
    }

    pub fn toggle_diff_edit_collapse(&mut self, path: &str, cx: &mut Context<Self>) {
        self.diff_edit.session.toggle_collapse(path);
        self.invalidate_diff_edit_rows(cx);
    }

    pub fn collapse_all_diff_edit(&mut self, cx: &mut Context<Self>) {
        let Some(files) = self.vm.read(cx).files.clone() else {
            return;
        };
        let paths: Vec<String> = files.iter().map(|hunk| hunk.path.clone()).collect();
        self.diff_edit.session.collapse_all(&paths);
        self.invalidate_diff_edit_rows(cx);
    }

    pub fn expand_all_diff_edit(&mut self, cx: &mut Context<Self>) {
        self.diff_edit.session.expand_all();
        self.invalidate_diff_edit_rows(cx);
    }

    pub fn diff_edit_collapsed(&self, path: &str) -> bool {
        self.diff_edit.session.is_collapsed(path)
    }

    pub fn diff_edit_stats_ready(&self) -> bool {
        self.diff_edit.stats.is_some()
    }

    pub(super) fn spawn_diff_edit_stats(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            return;
        };
        // The immutable commit id is the query revision: a change-id query could read an amended replacement that the completion guard still attributes to the on-screen commit.
        let Some(commit) = self.selected_commit_id(cx) else {
            return;
        };
        self.diff_edit.stats_commit = Some(commit.clone());
        let epoch = self.diff_edit.epoch;
        let ignore_whitespace = self.vm.read(cx).ignore_whitespace;
        cx.spawn(async move |this, cx| {
            let rev = commit.clone();
            let stats = cx
                .background_spawn(async move { repo.diff_file_stats(&rev, ignore_whitespace) })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.finish_diff_edit_stats(epoch, commit, ignore_whitespace, stats, cx)
            });
        })
        .detach();
    }

    fn finish_diff_edit_stats(
        &mut self,
        epoch: u64,
        commit: String,
        ignore_whitespace: bool,
        stats: CoreResult<Vec<FileDiffStats>>,
        cx: &mut Context<Self>,
    ) {
        if !self.diff_edit.active || self.diff_edit.epoch != epoch {
            return;
        }
        // The immutable commit id and captured mode must still match the displayed diff.
        if self.selected_commit_id(cx).as_deref() != Some(commit.as_str())
            || self.vm.read(cx).ignore_whitespace != ignore_whitespace
        {
            return;
        }
        let Ok(stats) = stats else {
            return;
        };
        let paths: Vec<String> = self
            .vm
            .read(cx)
            .files
            .as_ref()
            .map(|files| files.iter().map(|hunk| hunk.path.clone()).collect())
            .unwrap_or_default();
        self.diff_edit.session.apply_stats(&paths, &stats);
        self.diff_edit.stats = Some(stats.into_iter().map(|s| (s.path.clone(), s)).collect());
        self.invalidate_diff_edit_rows(cx);
    }
}
