use std::sync::Arc;

use gpui::Context;
use jayjay_core::dag::DagLayout;
use jayjay_core::{DEFAULT_REVSET_DEPTH, build_default_revset};

use super::view_model::RepoViewModel;
use crate::diff::{DetailMode, DiffViewMode};

impl RepoViewModel {
    pub(crate) fn toggle_view_mode(&mut self, cx: &mut Context<Self>) {
        self.view_mode = match self.view_mode {
            DiffViewMode::Unified => DiffViewMode::SideBySide,
            DiffViewMode::SideBySide => DiffViewMode::Unified,
        };
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn toggle_ignore_whitespace(&mut self, cx: &mut Context<Self>) {
        self.ignore_whitespace = !self.ignore_whitespace;
        let rev = self.selected_revision();
        let hunk = self
            .files
            .as_ref()
            .and_then(|f| self.selected_file_ix.and_then(|ix| f.get(ix).cloned()));
        if let (Some(rev), Some(hunk)) = (rev, hunk) {
            self.load_diff_async(rev, hunk, cx);
        } else {
            cx.notify();
        }
    }

    pub fn load_more(&mut self, cx: &mut Context<Self>) {
        if !self.can_load_more || !self.revset_is_default() {
            return;
        }
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let new_depth = self.revset_depth + DEFAULT_REVSET_DEPTH;
        let new_revset = build_default_revset(new_depth);
        let previous_ids: std::collections::HashSet<_> = self
            .graph
            .changes
            .iter()
            .map(|change| change.commit_id.id.clone())
            .collect();
        self.loading.more = true;
        self.can_load_more = false;
        self.clear_error();
        self.begin_refreshing(cx);
        self.loading.refresh_gen = self.loading.refresh_gen.wrapping_add(1);
        let generation = self.loading.refresh_gen;

        Self::background_update(
            cx,
            async move { repo.log_graph(&new_revset) },
            move |vm, result, cx| {
                vm.loading.more = false;
                vm.finish_repo_task(cx);
                if vm.loading.refresh_gen != generation {
                    return;
                }
                match result {
                    Ok(entries) => {
                        let did_grow = entries
                            .iter()
                            .any(|entry| !previous_ids.contains(&entry.change.commit_id.id));
                        vm.graph.dag_layout = Arc::new(DagLayout::compute(&entries));
                        vm.graph.changes =
                            Arc::new(entries.iter().map(|e| e.change.clone()).collect::<Vec<_>>());
                        vm.graph.entries = Arc::new(entries);
                        vm.can_load_more = did_grow && vm.graph.changes.len() >= new_depth as usize;
                        if did_grow {
                            vm.revset_depth = new_depth;
                            vm.revset = build_default_revset(new_depth).into();
                        }
                    }
                    Err(error) => vm.present_error(error),
                }
                cx.notify();
            },
        );
    }

    pub(crate) fn toggle_annotate(&mut self, cx: &mut Context<Self>) {
        self.detail_mode = match self.detail_mode {
            DetailMode::Annotate => DetailMode::Diff,
            DetailMode::Diff => DetailMode::Annotate,
        };
        if matches!(self.detail_mode, DetailMode::Annotate) {
            self.load_annotate(cx);
        }
        cx.notify();
    }
}
