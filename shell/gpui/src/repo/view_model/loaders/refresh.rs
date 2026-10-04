use std::sync::Arc;
use std::time::Duration;

use gpui::{Context, SharedString};
use jayjay_core::dag::DagLayout;
use jayjay_core::{
    BookmarkInfo, ChangeInfo, CoreError, CoreResult, DiffStats, GraphEntry, Repo, RevsetVocabulary,
    WorkspaceInfo,
};

use super::super::{PendingRefresh, RepoViewModel};

const MUTATION_ECHO_WINDOW: Duration = Duration::from_secs(5);

impl RepoViewModel {
    pub fn handle_operation_change(&mut self, cx: &mut Context<Self>) {
        self.loading
            .pending_auto_refresh
            .get_or_insert(PendingRefresh::CheckOperation);
        self.resume_pending_refresh(cx);
    }

    pub fn handle_working_copy_change(&mut self, cx: &mut Context<Self>) {
        // A suspended event must survive even if a mutation stamps the echo window before the overlay closes.
        if !self.refresh_suspended && self.is_internal_mutation_echo() {
            return;
        }
        self.loading.pending_auto_refresh = Some(PendingRefresh::Reload);
        self.resume_pending_refresh(cx);
    }

    /// The owed refresh runs without an echo re-check: the deferred event was external when it arrived.
    pub fn set_refresh_suspended(&mut self, suspended: bool, cx: &mut Context<Self>) {
        if self.refresh_suspended == suspended {
            return;
        }
        self.refresh_suspended = suspended;
        self.resume_pending_refresh(cx);
    }

    /// The at-head check waits for in-flight work: that refresh is what moves the loaded repo to the head it compares against.
    pub(crate) fn resume_pending_refresh(&mut self, cx: &mut Context<Self>) {
        if self.refresh_suspended || self.loading.refreshing {
            return;
        }
        let Some(pending) = self.loading.pending_auto_refresh.take() else {
            return;
        };
        if pending == PendingRefresh::CheckOperation
            && let Some(repo) = self.repo.as_ref()
            && repo.is_at_operation_head().unwrap_or(false)
        {
            return;
        }
        self.refresh(true, cx);
    }

    pub(in crate::repo) fn is_internal_mutation_echo(&self) -> bool {
        self.last_internal_mutation_at
            .is_some_and(|at| at.elapsed() < MUTATION_ECHO_WINDOW)
    }

    pub fn refresh(&mut self, is_auto_triggered: bool, cx: &mut Context<Self>) {
        let selection = self
            .selected
            .and_then(|ix| self.graph.changes.get(ix))
            .map(|c| (c.change_id.id.clone(), c.commit_id.id.clone()));
        self.refresh_preferring(is_auto_triggered, selection, cx);
    }

    /// `selection` is (change id, commit id): the commit wins, the change id is the fallback once a rewrite retired that commit.
    pub(in crate::repo::view_model) fn refresh_preferring(
        &mut self,
        is_auto_triggered: bool,
        selection: Option<(String, String)>,
        cx: &mut Context<Self>,
    ) {
        // FS event mid-refresh: defer it and re-run from the completion so the user's latest write isn't lost.
        if is_auto_triggered && self.loading.refreshing {
            self.loading.pending_auto_refresh = Some(PendingRefresh::Reload);
            return;
        }
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.pending_auto_refresh = None;
        // A background refresh must not dismiss an error the user is still reading; manual refresh is an explicit retry.
        if !is_auto_triggered {
            self.clear_error();
        }
        self.begin_refreshing(cx);
        self.loading.refresh_gen = self.loading.refresh_gen.wrapping_add(1);
        let generation = self.loading.refresh_gen;
        let revset = self.revset().to_owned();
        let previous_selection = selection;

        Self::background_update(
            cx,
            async move { refresh_graph_blocking(&repo, &revset) },
            move |vm, result, cx| {
                vm.finish_repo_task(cx);
                if vm.loading.refresh_gen != generation {
                    return;
                }
                // An overlay opened mid-flight: don't rewrite selection or detail under it; the gate owes a rerun on close.
                if is_auto_triggered && vm.refresh_suspended {
                    vm.loading.pending_auto_refresh = Some(PendingRefresh::Reload);
                    return;
                }
                // An FS event arrived after our snapshot, so this result may already be stale.
                if vm.loading.pending_auto_refresh.is_some() {
                    // A failed load may have left the repo behind the head, so its at-head answer proves nothing.
                    if result.is_err() {
                        vm.loading.pending_auto_refresh = Some(PendingRefresh::Reload);
                    }
                    vm.resume_pending_refresh(cx);
                    if vm.loading.refreshing {
                        return;
                    }
                }
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
        let pending_error = self.pending_error.take();
        match result {
            Ok(data) => {
                self.working_copy_stale = data.working_copy_stale;
                let entries = data.entries;
                self.fix_unavailable_reason = data.fix_unavailable_reason.map(SharedString::from);
                self.can_load_more = self
                    .revset_depth()
                    .is_some_and(|depth| entries.len() >= depth as usize);
                self.graph.bookmarks = Arc::new(data.bookmarks);
                self.vocabulary = data.vocabulary;
                if let Some(workspaces) = data.workspaces {
                    self.graph.workspaces = Arc::new(workspaces);
                }
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
                    // Keep the user's place in the file column across a background reload.
                    if self.pending_file_selection.is_none() {
                        self.pending_file_selection = self
                            .selected_file_ix
                            .and_then(|file_ix| self.files.as_ref()?.get(file_ix))
                            .map(|file| file.path.clone());
                    }
                    self.select_change(ix, cx);
                } else {
                    self.loading.change_gen = self.loading.change_gen.wrapping_add(1);
                    self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
                    self.selected = None;
                    self.selected_changes.clear();
                    self.clear_detail_state();
                    self.compare = None;
                    self.pr_info = None;
                }
                if pending_error.is_some() {
                    self.error = pending_error;
                }
            }
            Err(error) => self.present_error(error),
        }
        cx.notify();
    }
}

struct RefreshData {
    working_copy_stale: bool,
    entries: Vec<GraphEntry>,
    bookmarks: Vec<BookmarkInfo>,
    vocabulary: RevsetVocabulary,
    workspaces: Option<Vec<WorkspaceInfo>>,
    pr_host_name: Option<String>,
    working_copy_stats: Option<DiffStats>,
    current_operation_description: String,
    fix_unavailable_reason: Option<String>,
}

fn refresh_graph_blocking(repo: &Repo, revset: &str) -> CoreResult<RefreshData> {
    let working_copy_stale = match repo.refresh_working_copy() {
        Ok(()) => false,
        Err(CoreError::WorkingCopyStale) => true,
        Err(error) => return Err(error),
    };
    let entries = repo.log_graph(revset)?;
    let bookmarks = repo.list_bookmarks().unwrap_or_default();
    let vocabulary = repo.revset_vocabulary(&bookmarks);
    let workspaces = repo.workspace_list().ok();
    let pr_host_name = repo.pr_host_name();
    let working_copy_stats = repo.diff_stats("@").ok();
    let current_operation_description = repo.current_operation_description();
    let fix_unavailable_reason = repo.fix_unavailable_reason();
    Ok(RefreshData {
        working_copy_stale,
        entries,
        bookmarks,
        vocabulary,
        workspaces,
        pr_host_name,
        working_copy_stats,
        current_operation_description,
        fix_unavailable_reason,
    })
}
