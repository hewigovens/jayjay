use std::sync::Arc;

use gpui::Context;
use jayjay_core::dag::DagLayout;
use jayjay_core::{DEFAULT_REVSET_DEPTH, build_default_revset};

use super::view_model::RepoViewModel;
use crate::diff::{DetailMode, DiffViewMode};

impl RepoViewModel {
    pub fn toggle_view_mode(&mut self, cx: &mut Context<Self>) {
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
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let new_depth = self.revset_depth + DEFAULT_REVSET_DEPTH;
        self.revset_depth = new_depth;
        self.loading.more = true;
        self.clear_error();
        cx.notify();

        Self::background_update(
            cx,
            async move { repo.log_graph(&build_default_revset(new_depth)) },
            move |vm, result, cx| {
                vm.loading.more = false;
                match result {
                    Ok(entries) => {
                        vm.graph.dag_layout = Arc::new(DagLayout::compute(&entries));
                        vm.graph.changes =
                            Arc::new(entries.iter().map(|e| e.change.clone()).collect::<Vec<_>>());
                        vm.graph.entries = Arc::new(entries);
                    }
                    Err(error) => vm.present_error(error),
                }
                cx.notify();
            },
        );
    }

    pub fn toggle_annotate(&mut self, cx: &mut Context<Self>) {
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
