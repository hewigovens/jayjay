use std::sync::Arc;

use gpui::{AppContext, Context};

use super::RepoViewModel;
use crate::repo::revset::{self, CompareState, PrDiffRequest};

impl RepoViewModel {
    pub fn select_change(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
        let generation = self.loading.change_gen;
        self.compare = None;
        self.clear_error();
        self.selected = Some(ix);
        self.selected_file_ix = None;
        self.files = None;
        self.current_diff = None;
        self.diff_cache.clear();
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
        let rev = revset::change_revision(&change);

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
                match detail {
                    Ok(detail) => {
                        let files = Arc::new(detail.diff);
                        vm.files = Some(files.clone());
                        if !files.is_empty() {
                            vm.selected_file_ix = Some(0);
                            let hunk = files[0].clone();
                            vm.load_diff_async(rev, hunk, cx);
                            vm.preload_diffs_async(files, cx);
                        }
                    }
                    Err(error) => {
                        vm.files = None;
                        vm.selected_file_ix = None;
                        vm.present_error(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn select_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.selected_file_ix == Some(ix) {
            cx.notify();
            return;
        }

        self.selected_file_ix = Some(ix);
        self.clear_error();
        let rev = self.selected_revision();
        let hunk = self.files.as_ref().and_then(|f| f.get(ix)).cloned();
        if let (Some(rev), Some(hunk)) = (rev, hunk) {
            self.load_diff_async(rev, hunk, cx);
        } else {
            cx.notify();
        }
    }

    pub fn compare_pr_diff(&mut self, request: PrDiffRequest, cx: &mut Context<Self>) {
        self.compare_summary(request.compare_state(), cx);
    }

    pub fn compare_changes(&mut self, from_ix: usize, to_ix: usize, cx: &mut Context<Self>) {
        let (Some(from), Some(to)) = (
            self.graph.changes.get(from_ix).cloned(),
            self.graph.changes.get(to_ix).cloned(),
        ) else {
            return;
        };
        if let Some(request) = revset::pr_diff_request(&from, &to) {
            let mut compare = request.compare_state();
            compare.source_change_id = Some(from.change_id.clone());
            self.compare_summary(compare, cx);
            return;
        }
        self.compare_summary(revset::compare_state_between(&from, &to), cx);
    }

    pub fn reverse_compare(&mut self, cx: &mut Context<Self>) {
        let Some(compare) = self.compare.clone() else {
            return;
        };
        let (Some(source_id), Some(target_id)) = (
            compare.source_change_id.as_deref(),
            compare.target_change_id.as_deref(),
        ) else {
            return;
        };
        let source = self
            .graph
            .changes
            .iter()
            .find(|change| change.change_id == source_id)
            .cloned();
        let target = self
            .graph
            .changes
            .iter()
            .find(|change| change.change_id == target_id)
            .cloned();
        let (Some(source), Some(target)) = (source, target) else {
            return;
        };

        if let Some(request) = revset::pr_diff_request(&target, &source) {
            let mut next = request.compare_state();
            next.source_change_id = Some(target.change_id.clone());
            self.compare_summary(next, cx);
            return;
        }

        self.compare_summary(revset::compare_state_between(&target, &source), cx);
    }

    fn compare_summary(&mut self, compare: CompareState, cx: &mut Context<Self>) {
        self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
        let generation = self.loading.change_gen;
        let from_rev = compare.from_rev.clone();
        let to_rev = compare.to_rev.clone();
        let target_change_id = compare.target_change_id.clone();

        self.clear_error();
        self.compare = Some(compare);
        self.selected = target_change_id.and_then(|target_id| {
            self.graph
                .changes
                .iter()
                .position(|change| change.change_id == target_id)
        });
        self.selected_file_ix = None;
        self.files = None;
        self.current_diff = None;
        self.change_stats = None;
        self.loading.files = true;
        self.loading.diff = false;
        self.pr_info = None;
        self.loading.wc_changes = false;
        cx.notify();

        let Some(repo) = self.repo.clone() else {
            self.loading.files = false;
            cx.notify();
            return;
        };

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn({
                    let repo = repo.clone();
                    let from_rev = from_rev.clone();
                    let to_rev = to_rev.clone();
                    async move { repo.interdiff_summary(&from_rev, &to_rev) }
                })
                .await;

            let _ = this.update(cx, move |vm, cx| {
                if vm.loading.change_gen != generation {
                    return;
                }
                vm.loading.files = false;
                match result {
                    Ok(detail) => {
                        let files = Arc::new(detail.diff);
                        vm.files = Some(files.clone());
                        if !files.is_empty() {
                            vm.selected_file_ix = Some(0);
                            let hunk = files[0].clone();
                            vm.load_diff_async(to_rev, hunk, cx);
                        }
                    }
                    Err(error) => {
                        vm.compare = None;
                        vm.files = None;
                        vm.selected_file_ix = None;
                        vm.present_error(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn clear_compare(&mut self, cx: &mut Context<Self>) {
        let selected = self.selected;
        self.compare = None;
        if let Some(ix) = selected {
            self.select_change(ix, cx);
        } else {
            cx.notify();
        }
    }
}
