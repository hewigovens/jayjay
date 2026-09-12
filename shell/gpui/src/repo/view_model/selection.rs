use std::sync::Arc;

use gpui::Context;
use jayjay_core::ChangeInfo;
use jayjay_core::dag::{SelectionGraph, SelectionState};

use super::{RepoViewModel, SelectionCache};
use crate::repo::revset::{self, BookmarkDiffRequest, CompareState};
use crate::ui::ordered_selection::SelectionClick;

impl RepoViewModel {
    /// Preserve `revision` through the next refresh by resolving it in the current graph; `None` deliberately lets refresh fall back to the working copy.
    pub(super) fn refresh_selecting_revision(
        &mut self,
        revision: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        self.selected = revision.and_then(|revision| {
            self.graph.changes.iter().position(|change| {
                change.change_id.id == revision || change.commit_id.id == revision
            })
        });
        self.refresh(false, cx);
    }

    pub fn select_change(&mut self, ix: usize, cx: &mut Context<Self>) {
        // Consumed synchronously, not lazily in the async completion below, so a superseded `select_change` can't leave this set for an unrelated later selection to pick up.
        let restore_path = self.pending_file_selection.take();
        self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
        let generation = self.loading.change_gen;
        self.compare = None;
        self.clear_error();
        self.selected = Some(ix);
        self.selected_changes.replace(ix);
        self.clear_detail_state();
        self.loading.files = true;
        // Bump pr_gen so a stale fetch from the prior selection can't overwrite this reset, even when the new change has no bookmark to trigger refresh_pr_info.
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        self.pr_info = None;

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
                    let conflicted_paths = repo
                        .resolve_list(&rev)
                        .unwrap_or_default()
                        .into_iter()
                        .collect();
                    (detail, stats, conflicted_paths)
                }
            },
            move |vm, (detail, stats, conflicted_paths), cx| {
                // Drop stale results from a superseded select_change.
                if vm.loading.change_gen != generation {
                    return;
                }
                vm.loading.files = false;
                vm.change_stats = stats;
                vm.conflicted_paths = conflicted_paths;
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

    pub(crate) fn select_file(&mut self, ix: usize, cx: &mut Context<Self>) {
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

    pub(crate) fn compare_bookmark_diff(
        &mut self,
        request: BookmarkDiffRequest,
        cx: &mut Context<Self>,
    ) {
        self.selected_changes.clear();
        self.compare_summary(request.compare_state(), cx);
    }

    pub(crate) fn update_change_selection(
        &mut self,
        ix: usize,
        click: SelectionClick,
        cx: &mut Context<Self>,
    ) {
        if ix >= self.graph.changes.len() {
            return;
        }
        let order: Vec<_> = (0..self.graph.changes.len()).collect();
        self.selected_changes.apply(click, ix, &order);
        let selected = self.selected_change_indices();
        let changes: Vec<_> = selected
            .iter()
            .filter_map(|ix| self.graph.changes.get(*ix).cloned())
            .collect();
        match selected.as_slice() {
            [] => self.show_selection_without_diff(None, cx),
            [only] => self.select_change(*only, cx),
            _ if !self.selected_changes.is_contiguous_in(&order) => {
                if let (Some(newest), Some(oldest)) = (changes.first(), changes.last()) {
                    self.compare_summary(revset::compare_state_between(oldest, newest), cx);
                }
            }
            _ if self.has_diffable_linear_selection() => {
                if let Some(compare) = revset::combined_compare_state(&changes) {
                    self.compare_summary(compare, cx);
                }
            }
            _ => self.show_selection_without_diff(self.selected_changes.primary().copied(), cx),
        }
    }

    fn show_selection_without_diff(&mut self, selected: Option<usize>, cx: &mut Context<Self>) {
        self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        self.selected = selected;
        self.compare = None;
        self.pr_info = None;
        self.clear_detail_state();
        cx.notify();
    }

    pub(crate) fn reverse_compare(&mut self, cx: &mut Context<Self>) {
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
            self.graph.changes.iter().position(|change| {
                change.change_id.id == target_id || change.commit_id.id == target_id
            })
        });
        if self.selected_changes.len() <= 1 {
            if let Some(selected) = self.selected {
                self.selected_changes.replace(selected);
            } else {
                self.selected_changes.clear();
            }
        }
        self.clear_detail_state();
        self.loading.files = true;
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        self.pr_info = None;
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
            self.clear_detail_state();
            cx.notify();
        }
    }

    pub fn selected_change_indices(&self) -> Vec<usize> {
        let order: Vec<_> = (0..self.graph.changes.len()).collect();
        self.selected_changes.ordered(&order)
    }

    pub fn has_multiple_change_selection(&self) -> bool {
        self.selected_changes.len() > 1
    }

    pub(crate) fn multi_selection_primary_index(&self) -> Option<usize> {
        self.selected_changes
            .primary()
            .copied()
            .filter(|_| self.has_multiple_change_selection())
    }

    pub fn selection_without_diff_count(&self) -> Option<usize> {
        (self.has_multiple_change_selection() && self.compare.is_none())
            .then_some(self.selected_changes.len())
    }

    pub fn is_change_selected(&self, ix: usize) -> bool {
        self.selected_changes.contains(&ix)
    }

    pub fn selected_revisions(&self) -> Vec<String> {
        self.selected_change_indices()
            .into_iter()
            .filter_map(|ix| self.graph.changes.get(ix))
            .map(revset::change_revision)
            .collect()
    }

    pub fn can_abandon_selected_changes(&self) -> bool {
        self.selection_state().can_abandon
    }

    pub fn can_squash_selected_changes(&self) -> bool {
        self.selection_state().can_squash
    }

    fn has_diffable_linear_selection(&self) -> bool {
        self.selection_state().can_diff
    }

    pub fn can_merge_selected_changes(&self) -> bool {
        self.selection_state().can_merge
    }

    pub fn can_merge_selected_change_with(&self, target: &ChangeInfo) -> bool {
        self.row_of(target)
            .is_some_and(|ix| self.selection_state().can_merge_with[ix])
    }

    pub fn can_rebase_selected_changes_onto(&self, target_ix: usize) -> bool {
        self.selection_state()
            .can_rebase_onto
            .get(target_ix)
            .copied()
            .unwrap_or(false)
    }

    fn row_of(&self, change: &ChangeInfo) -> Option<usize> {
        self.graph
            .changes
            .iter()
            .position(|candidate| candidate.commit_id == change.commit_id)
    }

    fn selection_state(&self) -> Arc<SelectionState> {
        let selected = self.selected_change_indices();
        let mut cache = self.selection_cache.borrow_mut();
        if cache
            .as_ref()
            .is_none_or(|cached| !Arc::ptr_eq(&cached.entries, &self.graph.entries))
        {
            *cache = Some(SelectionCache {
                entries: self.graph.entries.clone(),
                graph: SelectionGraph::new(&self.graph.entries),
                state: None,
            });
        }
        let cache = cache.as_mut().expect("selection cache installed above");
        if cache
            .state
            .as_ref()
            .is_none_or(|(cached, _)| cached != &selected)
        {
            let commit_ids: Vec<String> = selected
                .iter()
                .filter_map(|&ix| self.graph.changes.get(ix))
                .map(|change| change.commit_id.id.clone())
                .collect();
            let state = Arc::new(cache.graph.state(&commit_ids));
            cache.state = Some((selected, state));
        }
        cache
            .state
            .as_ref()
            .expect("state computed above")
            .1
            .clone()
    }

    pub(super) fn clear_detail_state(&mut self) {
        self.loading.diff_gen = self.loading.diff_gen.wrapping_add(1);
        self.selected_file_ix = None;
        self.files = None;
        self.conflicted_paths.clear();
        self.current_diff = None;
        self.current_projection = None;
        self.current_svg_preview = None;
        self.current_markdown_preview = None;
        self.current_diff_old_content = None;
        self.current_diff_new_content = None;
        self.current_diff_supports_file_editor = false;
        self.clear_diff_cache_state();
        self.change_stats = None;
        self.loading.files = false;
        self.loading.diff = false;
    }
}
