use std::sync::Arc;

use gpui::Context;

use super::RepoViewModel;
use crate::repo::revset::{self, BookmarkDiffRequest, CompareState};

impl RepoViewModel {
    pub fn select_change(&mut self, ix: usize, cx: &mut Context<Self>) {
        // Selecting the WC row while badged must re-snapshot rather than render the stale snapshot.
        if self.loading.wc_changes
            && self.compare.is_none()
            && self
                .graph
                .changes
                .get(ix)
                .is_some_and(|c| c.is_working_copy)
        {
            self.refresh(true, cx);
            return;
        }
        // Consumed synchronously, not lazily in the async completion below, so a superseded `select_change` can't leave this set for an unrelated later selection to pick up.
        let restore_path = self.pending_file_selection.take();
        self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
        let generation = self.loading.change_gen;
        self.compare = None;
        self.clear_error();
        self.selected = Some(ix);
        self.selected_file_ix = None;
        self.files = None;
        self.current_diff = None;
        self.current_projection = None;
        self.current_svg_preview = None;
        self.current_markdown_preview = None;
        self.current_diff_old_content = None;
        self.current_diff_new_content = None;
        self.clear_diff_cache_state();
        self.change_stats = None;
        self.loading.files = true;
        self.loading.diff = false;
        // Bump pr_gen so a stale fetch from the prior selection can't overwrite this reset, even when the new change has no bookmark to trigger refresh_pr_info.
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        self.pr_info = None;
        // Keep `wc_changes`: selecting a row doesn't re-snapshot, so the staleness badge survives until a refresh.

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

        Self::background_update(
            cx,
            {
                let repo = repo.clone();
                let rev = rev.clone();
                async move {
                    let detail = repo.show_summary(&rev);
                    let stats = repo.diff_stats(&rev).ok();
                    (detail, stats)
                }
            },
            move |vm, (detail, stats), cx| {
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
                            let ix = restore_path
                                .as_ref()
                                .and_then(|path| files.iter().position(|f| &f.path == path))
                                .unwrap_or(0);
                            vm.selected_file_ix = Some(ix);
                            let hunk = files[ix].clone();
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
            },
        );
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

    pub fn compare_bookmark_diff(&mut self, request: BookmarkDiffRequest, cx: &mut Context<Self>) {
        self.compare_summary(request.compare_state(), cx);
    }

    pub fn compare_changes(&mut self, from_ix: usize, to_ix: usize, cx: &mut Context<Self>) {
        let (Some(from), Some(to)) = (
            self.graph.changes.get(from_ix).cloned(),
            self.graph.changes.get(to_ix).cloned(),
        ) else {
            return;
        };
        if let Some(request) = revset::bookmark_diff_request(&from, &to) {
            let mut compare = request.compare_state();
            compare.source_change_id = Some(from.change_id.id.clone());
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
            .find(|change| change.change_id.id == source_id)
            .cloned();
        let target = self
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == target_id)
            .cloned();
        let (Some(source), Some(target)) = (source, target) else {
            return;
        };

        if let Some(request) = revset::bookmark_diff_request(&target, &source) {
            let mut next = request.compare_state();
            next.source_change_id = Some(target.change_id.id.clone());
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
        self.current_projection = None;
        self.current_svg_preview = None;
        self.current_markdown_preview = None;
        self.current_diff_old_content = None;
        self.current_diff_new_content = None;
        self.clear_diff_cache_state();
        self.change_stats = None;
        self.loading.files = true;
        self.loading.diff = false;
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        self.pr_info = None;
        // Comparing two revs doesn't re-snapshot the WC; keep the staleness badge until a refresh.
        cx.notify();

        let Some(repo) = self.repo.clone() else {
            self.loading.files = false;
            cx.notify();
            return;
        };

        Self::background_update(
            cx,
            {
                let repo = repo.clone();
                let from_rev = from_rev.clone();
                let to_rev = to_rev.clone();
                async move { repo.interdiff_summary(&from_rev, &to_rev) }
            },
            move |vm, result, cx| {
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
            },
        );
    }

    pub fn clear_compare(&mut self, cx: &mut Context<Self>) {
        self.compare = None;
        let fallback = self.selected.or_else(|| {
            self.graph
                .changes
                .iter()
                .position(|change| change.is_working_copy)
                .or_else(|| (!self.graph.changes.is_empty()).then_some(0))
        });
        if let Some(ix) = fallback {
            self.select_change(ix, cx);
        } else {
            self.selected = None;
            self.selected_file_ix = None;
            self.files = None;
            self.current_diff = None;
            self.current_projection = None;
            self.current_svg_preview = None;
            self.current_markdown_preview = None;
            self.change_stats = None;
            self.loading.files = false;
            self.loading.diff = false;
            self.clear_diff_cache_state();
            cx.notify();
        }
    }
}
