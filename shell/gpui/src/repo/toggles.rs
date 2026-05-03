use std::sync::Arc;

use gpui::{AppContext, Context};
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
        let rev = self
            .selected
            .and_then(|c| self.graph.changes.get(c))
            .map(|c| c.change_id.clone());
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
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.log_graph(&build_default_revset(new_depth)) })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                vm.loading.more = false;
                if let Ok(entries) = result {
                    vm.graph.dag_layout = Arc::new(DagLayout::compute(&entries));
                    vm.graph.changes =
                        Arc::new(entries.iter().map(|e| e.change.clone()).collect::<Vec<_>>());
                    vm.graph.entries = Arc::new(entries);
                }
                cx.notify();
            });
        })
        .detach();
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
