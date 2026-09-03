use std::sync::Arc;

use gpui::Context;
use jayjay_core::{CoreResult, LOG_PAGE_SIZE, LogGraphPage};

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
        if !self.can_load_more {
            return;
        }
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let query = self.current_log_query();
        let new_limit = self.applied_limit + LOG_PAGE_SIZE;
        self.loading.more = true;
        self.can_load_more = false;
        self.clear_error();
        self.begin_refreshing(cx);
        self.loading.refresh_gen = self.loading.refresh_gen.wrapping_add(1);
        let generation = self.loading.refresh_gen;

        Self::background_update(
            cx,
            async move { repo.log_graph_page(&query, new_limit) },
            move |vm, result: CoreResult<LogGraphPage>, cx| {
                vm.loading.more = false;
                vm.finish_repo_task(cx);
                if vm.loading.refresh_gen != generation {
                    return;
                }
                match result {
                    Ok(page) => {
                        vm.graph.dag_layout = Arc::new(page.layout);
                        vm.graph.changes = Arc::new(
                            page.entries
                                .iter()
                                .map(|e| e.change.clone())
                                .collect::<Vec<_>>(),
                        );
                        vm.graph.entries = Arc::new(page.entries);
                        vm.applied_limit = new_limit;
                        vm.can_load_more = page.has_more;
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
