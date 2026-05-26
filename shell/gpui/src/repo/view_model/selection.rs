use std::sync::Arc;

use gpui::{AppContext, Context};

use super::RepoViewModel;

impl RepoViewModel {
    pub fn select_change(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
        let generation = self.loading.change_gen;
        self.selected = Some(ix);
        self.selected_file_ix = None;
        self.files = None;
        self.current_diff = None;
        self.change_stats = None;
        self.loading.files = true;
        self.loading.diff = false;
        self.pr_info = None;
        self.loading.wc_changes = false;

        if let Some(change) = self.graph.changes.get(ix).cloned() {
            self.ensure_avatar(change.author.email.clone(), cx);
            self.refresh_pr_info(&change, cx);
        }

        let (Some(repo), Some(change)) = (self.repo.clone(), self.graph.changes.get(ix).cloned())
        else {
            self.loading.files = false;
            cx.notify();
            return;
        };
        let rev = change.change_id.clone();

        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn({
                    let repo = repo.clone();
                    let rev = rev.clone();
                    async move {
                        let detail = repo.show_summary(&rev);
                        let stats = repo.diff_stats(&rev).ok();
                        (detail, stats)
                    }
                })
                .await;
            let (detail, stats) = result;

            let _ = this.update(cx, move |vm, cx| {
                // Drop stale results from a superseded select_change.
                if vm.loading.change_gen != generation {
                    return;
                }
                vm.loading.files = false;
                vm.change_stats = stats;
                if let Ok(detail) = detail {
                    let files = Arc::new(detail.diff);
                    vm.files = Some(files.clone());
                    if !files.is_empty() {
                        vm.selected_file_ix = Some(0);
                        let hunk = files[0].clone();
                        vm.load_diff_async(rev, hunk, cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn select_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected_file_ix = Some(ix);
        let rev = self
            .selected
            .and_then(|c| self.graph.changes.get(c))
            .map(|c| c.change_id.clone());
        let hunk = self.files.as_ref().and_then(|f| f.get(ix)).cloned();
        if let (Some(rev), Some(hunk)) = (rev, hunk) {
            self.load_diff_async(rev, hunk, cx);
        } else {
            cx.notify();
        }
    }
}
