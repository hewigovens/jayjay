mod diff;
mod diff_compute;
mod review_notes;

use std::sync::Arc;
use std::time::Duration;

use gpui::{Context, SharedString};
use jayjay_core::dag::DagLayout;
use jayjay_core::{
    BookmarkInfo, ChangeInfo, CoreResult, DiffStats, GraphEntry, Repo, WorkspaceInfo,
    build_default_revset,
};

use super::RepoViewModel;
use crate::repo::revset;

/// Window during which FS echoes from our own mutations are ignored.
const MUTATION_ECHO_WINDOW: Duration = Duration::from_secs(5);

impl RepoViewModel {
    pub(in crate::repo) fn load_annotate(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let Some(rev) = self
            .selected
            .and_then(|i| self.graph.changes.get(i))
            .map(revset::change_revision)
        else {
            return;
        };
        let Some(path) = self.selected_hunk().map(|h| h.path.clone()) else {
            return;
        };

        self.loading.annotate_gen = self.loading.annotate_gen.wrapping_add(1);
        let generation = self.loading.annotate_gen;
        self.annotate_lines = None;
        self.loading.annotate = true;
        cx.notify();

        Self::background_update(
            cx,
            async move { repo.annotate_file(&rev, &path).ok() },
            move |vm, result, cx| {
                if vm.loading.annotate_gen != generation {
                    return;
                }
                vm.loading.annotate = false;
                vm.annotate_lines = result.map(Arc::new);
                cx.notify();
            },
        );
    }

    pub(in crate::repo) fn refresh_pr_info(&mut self, change: &ChangeInfo, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let Some(bookmark) = change.bookmarks.first().cloned() else {
            return;
        };
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        let generation = self.loading.pr_gen;
        self.loading.pr = true;
        Self::background_update(
            cx,
            async move { repo.pull_request_info(&bookmark) },
            move |vm, info, cx| {
                // A newer selection's fetch superseded this one; its result lands later.
                if vm.loading.pr_gen != generation {
                    return;
                }
                vm.loading.pr = false;
                vm.pr_info = info;
                cx.notify();
            },
        );
    }

    pub fn handle_working_copy_change(&mut self, cx: &mut Context<Self>) {
        // Ignore the FS echo from our own mutations — the mutation path already refreshed.
        if self
            .last_internal_mutation_at
            .is_some_and(|at| at.elapsed() < MUTATION_ECHO_WINDOW)
        {
            return;
        }
        // While the user is actively reviewing the WC, just badge — don't yank the diff out.
        if self.is_repo_window_active
            && self.compare.is_none()
            && self.selected_change().is_some_and(|c| c.is_working_copy)
        {
            self.loading.wc_changes = true;
            // A badge set mid-refresh must survive the in-flight completion's clear.
            if self.loading.refreshing {
                self.loading.pending_auto_refresh = true;
            }
            cx.notify();
            return;
        }
        self.refresh(true, cx);
    }

    pub fn refresh(&mut self, is_auto_triggered: bool, cx: &mut Context<Self>) {
        // FS event mid-refresh: defer it and re-run from the completion so the user's latest write isn't lost.
        if is_auto_triggered && self.loading.refreshing {
            self.loading.pending_auto_refresh = true;
            return;
        }
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.pending_auto_refresh = false;
        self.clear_error();
        self.begin_refreshing(cx);
        self.loading.refresh_gen = self.loading.refresh_gen.wrapping_add(1);
        let generation = self.loading.refresh_gen;
        let depth = self.revset_depth;
        let previous_selection = self
            .selected
            .and_then(|ix| self.graph.changes.get(ix))
            .map(|c| (c.change_id.id.clone(), c.commit_id.id.clone()));

        Self::background_update(
            cx,
            async move { refresh_graph_blocking(&repo, depth) },
            move |vm, result, cx| {
                vm.finish_refreshing(cx);
                // A later refresh superseded this one; drop this stale result.
                if vm.loading.refresh_gen != generation {
                    return;
                }
                // An FS event arrived after our snapshot, so this result is already stale.
                if vm.loading.pending_auto_refresh {
                    vm.loading.pending_auto_refresh = false;
                    // Reviewing the WC: keep the badge. Otherwise re-run so the latest write isn't lost.
                    if vm.loading.wc_changes {
                        return;
                    }
                    vm.refresh(true, cx);
                    return;
                }
                vm.loading.wc_changes = false;
                vm.apply_refresh_result(result, previous_selection, cx);
            },
        );
    }

    fn apply_refresh_result(
        &mut self,
        result: CoreResult<RefreshData>,
        previous_selection: Option<(String, String)>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(data) => {
                let entries = data.entries;
                self.graph.bookmarks = Arc::new(data.bookmarks);
                self.graph.workspaces = Arc::new(data.workspaces);
                self.pr_host_name = data.pr_host_name.map(SharedString::from);
                self.working_copy_stats = data.working_copy_stats;
                self.current_operation_description = data.current_operation_description;
                self.graph.dag_layout = Arc::new(DagLayout::compute(&entries));
                let changes: Vec<ChangeInfo> = entries.iter().map(|e| e.change.clone()).collect();
                let new_selected = previous_selection
                    .as_ref()
                    .and_then(|(_, commit_id)| {
                        changes.iter().position(|c| &c.commit_id.id == commit_id)
                    })
                    .or_else(|| {
                        previous_selection.as_ref().and_then(|(change_id, _)| {
                            changes.iter().position(|c| &c.change_id.id == change_id)
                        })
                    })
                    .or_else(|| changes.iter().position(|c| c.is_working_copy))
                    .or(if changes.is_empty() { None } else { Some(0) });
                self.graph.changes = Arc::new(changes);
                self.graph.entries = Arc::new(entries);
                // Re-select even if the index is unchanged — file contents may have.
                if let Some(ix) = new_selected {
                    self.select_change(ix, cx);
                } else {
                    self.selected = None;
                }
            }
            Err(error) => self.present_error(error),
        }
        cx.notify();
    }

    pub fn ensure_avatar(&mut self, email: String, cx: &mut Context<Self>) {
        if email.trim().is_empty() {
            return;
        }
        if self.avatar_in_flight.contains(&email) {
            return;
        }
        if let Some(path) = crate::ui::avatar::cache_path(&email)
            && path.exists()
        {
            return;
        }
        self.avatar_in_flight.insert(email.clone());
        let email_for_remove = email.clone();
        Self::background_update(
            cx,
            async move {
                crate::ui::avatar::fetch_blocking(&email);
            },
            move |vm, (), cx| {
                vm.avatar_in_flight.remove(&email_for_remove);
                cx.notify();
            },
        );
    }
}

struct RefreshData {
    entries: Vec<GraphEntry>,
    bookmarks: Vec<BookmarkInfo>,
    workspaces: Vec<WorkspaceInfo>,
    pr_host_name: Option<String>,
    working_copy_stats: Option<DiffStats>,
    current_operation_description: String,
}

fn refresh_graph_blocking(repo: &Repo, depth: u32) -> CoreResult<RefreshData> {
    repo.refresh_working_copy()?;
    let entries = repo.log_graph(&build_default_revset(depth))?;
    let bookmarks = repo.list_bookmarks().unwrap_or_default();
    let workspaces = repo.workspace_list().unwrap_or_default();
    let pr_host_name = repo.pr_host_name();
    let working_copy_stats = repo.diff_stats("@").ok();
    let current_operation_description = repo.current_operation_description();
    Ok(RefreshData {
        entries,
        bookmarks,
        workspaces,
        pr_host_name,
        working_copy_stats,
        current_operation_description,
    })
}
