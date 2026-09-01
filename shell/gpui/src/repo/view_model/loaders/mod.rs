mod diff;
mod diff_compute;
mod review_notes;

use std::sync::Arc;
use std::time::Duration;

use gpui::{Context, SharedString};
use jayjay_core::dag::DagLayout;
use jayjay_core::{
    BookmarkInfo, ChangeInfo, CoreResult, DEFAULT_REVSET_DEPTH, DiffStats, GraphEntry, Repo,
    WorkspaceInfo, build_default_revset,
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

    pub fn handle_fs_event(&mut self, cx: &mut Context<Self>) {
        // Gate before the echo check: an event remembered here must survive even if a mutation stamps the echo window before the overlay closes.
        if self.refresh_suspended {
            self.loading.pending_auto_refresh = true;
            return;
        }
        // Ignore the FS echo from our own mutations — the mutation path already refreshed.
        if self.is_internal_mutation_echo() {
            return;
        }
        self.refresh(true, cx);
    }

    /// The owed refresh runs without an echo re-check: the deferred event was external when it arrived.
    pub fn set_refresh_suspended(&mut self, suspended: bool, cx: &mut Context<Self>) {
        if self.refresh_suspended == suspended {
            return;
        }
        self.refresh_suspended = suspended;
        if !suspended && self.loading.pending_auto_refresh {
            self.loading.pending_auto_refresh = false;
            self.refresh(true, cx);
        }
    }

    pub(in crate::repo) fn is_internal_mutation_echo(&self) -> bool {
        self.last_internal_mutation_at
            .is_some_and(|at| at.elapsed() < MUTATION_ECHO_WINDOW)
    }

    /// Refresh only the workspace picker without reloading the graph or selected change.
    pub(crate) fn refresh_workspaces(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.workspaces_gen = self.loading.workspaces_gen.wrapping_add(1);
        let generation = self.loading.workspaces_gen;
        Self::background_update(
            cx,
            async move { repo.workspace_list() },
            move |vm, workspaces, cx| {
                if vm.loading.workspaces_gen != generation {
                    return;
                }
                if let Ok(workspaces) = workspaces {
                    vm.graph.workspaces = Arc::new(workspaces);
                    cx.notify();
                }
            },
        );
    }

    pub fn refresh(&mut self, is_auto_triggered: bool, cx: &mut Context<Self>) {
        let selection = self
            .selected
            .and_then(|ix| self.graph.changes.get(ix))
            .map(|c| (c.change_id.id.clone(), c.commit_id.id.clone()));
        self.refresh_preferring(is_auto_triggered, selection, cx);
    }

    /// `selection` is (change id, commit id): the commit wins, the change id is the fallback once a rewrite retired that commit.
    pub(super) fn refresh_preferring(
        &mut self,
        is_auto_triggered: bool,
        selection: Option<(String, String)>,
        cx: &mut Context<Self>,
    ) {
        // FS event mid-refresh: defer it and re-run from the completion so the user's latest write isn't lost.
        if is_auto_triggered && self.loading.refreshing {
            self.loading.pending_auto_refresh = true;
            return;
        }
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.pending_auto_refresh = false;
        // A background refresh must not dismiss an error the user is still reading; manual refresh is an explicit retry.
        if !is_auto_triggered {
            self.clear_error();
        }
        self.begin_refreshing(cx);
        self.loading.refresh_gen = self.loading.refresh_gen.wrapping_add(1);
        let generation = self.loading.refresh_gen;
        let revset = self.revset.to_string();
        let previous_selection = selection;

        Self::background_update(
            cx,
            async move { refresh_graph_blocking(&repo, &revset) },
            move |vm, result, cx| {
                vm.finish_repo_task(cx);
                // A later refresh superseded this one; drop this stale result.
                if vm.loading.refresh_gen != generation {
                    return;
                }
                // An overlay opened mid-flight: don't rewrite selection or detail under it; the gate owes a rerun on close.
                if is_auto_triggered && vm.refresh_suspended {
                    vm.loading.pending_auto_refresh = true;
                    return;
                }
                // An FS event arrived after our snapshot, so this result is already stale.
                if vm.loading.pending_auto_refresh {
                    vm.loading.pending_auto_refresh = false;
                    vm.refresh(true, cx);
                    return;
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
        match result {
            Ok(data) => {
                let entries = data.entries;
                self.can_load_more =
                    self.revset_is_default() && entries.len() >= self.revset_depth as usize;
                self.graph.bookmarks = Arc::new(data.bookmarks);
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
                    // Keep the user's place in the file column across a background reload; mutation paths may have staked a restore target already.
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
            }
            Err(error) => self.present_error(error),
        }
        cx.notify();
    }

    pub fn apply_revset(&mut self, revset: &str, cx: &mut Context<Self>) {
        let trimmed = revset.trim();
        let default_revset = build_default_revset(DEFAULT_REVSET_DEPTH);
        if trimmed.is_empty() || trimmed == default_revset {
            self.revset_depth = DEFAULT_REVSET_DEPTH;
            self.revset = default_revset.into();
        } else {
            self.revset = trimmed.to_owned().into();
        }
        self.can_load_more = false;
        self.refresh(false, cx);
    }

    pub(crate) fn revset_is_default(&self) -> bool {
        self.revset.as_ref() == build_default_revset(self.revset_depth)
    }

    pub(crate) fn ensure_avatar(&mut self, email: String, cx: &mut Context<Self>) {
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
    workspaces: Option<Vec<WorkspaceInfo>>,
    pr_host_name: Option<String>,
    working_copy_stats: Option<DiffStats>,
    current_operation_description: String,
}

fn refresh_graph_blocking(repo: &Repo, revset: &str) -> CoreResult<RefreshData> {
    repo.refresh_working_copy()?;
    let entries = repo.log_graph(revset)?;
    let bookmarks = repo.list_bookmarks().unwrap_or_default();
    let workspaces = repo.workspace_list().ok();
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
