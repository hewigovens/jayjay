use std::sync::Arc;

use gpui::{AppContext, Context};
use jayjay_core::dag::DagLayout;
use jayjay_core::diff::{FileDiff, compute_file_diff};
use jayjay_core::{ChangeInfo, DiffHunk, Repo, build_default_revset};

use super::RepoViewModel;
use crate::diff::DetailMode;

impl RepoViewModel {
    pub(in crate::repo) fn load_diff_async(
        &mut self,
        rev: String,
        hunk: DiffHunk,
        cx: &mut Context<Self>,
    ) {
        self.loading.diff_gen = self.loading.diff_gen.wrapping_add(1);
        let generation = self.loading.diff_gen;
        self.current_diff = None;
        self.loading.diff = true;

        let Some(repo) = self.repo.clone() else {
            self.loading.diff = false;
            cx.notify();
            return;
        };
        let ignore_whitespace = self.ignore_whitespace;

        cx.notify();

        cx.spawn(async move |this, cx| {
            let file_diff = cx
                .background_spawn(async move {
                    compute_diff_blocking(&repo, &rev, &hunk, ignore_whitespace)
                })
                .await;

            let _ = this.update(cx, move |vm, cx| {
                if vm.loading.diff_gen != generation {
                    return;
                }
                vm.loading.diff = false;
                vm.current_diff = file_diff.map(Arc::new);
                if matches!(vm.detail_mode, DetailMode::Annotate) {
                    vm.load_annotate(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::repo) fn load_annotate(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let Some(rev) = self
            .selected
            .and_then(|i| self.graph.changes.get(i))
            .map(|c| c.change_id.clone())
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

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.annotate_file(&rev, &path).ok() })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                if vm.loading.annotate_gen != generation {
                    return;
                }
                vm.loading.annotate = false;
                vm.annotate_lines = result.map(Arc::new);
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::repo) fn refresh_pr_info(&mut self, change: &ChangeInfo, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let Some(bookmark) = change.bookmarks.first().cloned() else {
            return;
        };
        self.loading.pr = true;
        cx.spawn(async move |this, cx| {
            let info = cx
                .background_spawn(async move { repo.gh_pr_info(&bookmark) })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                vm.loading.pr = false;
                vm.pr_info = info;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn refresh(&mut self, is_auto_triggered: bool, cx: &mut Context<Self>) {
        // Skip FS-triggered re-entry; our own jj reads write back to op_heads → loop.
        if is_auto_triggered && self.loading.refreshing {
            return;
        }
        // While the user is reviewing the WC, just badge — don't yank the diff out.
        if is_auto_triggered && self.selected_change().is_some_and(|c| c.is_working_copy) {
            self.loading.wc_changes = true;
            cx.notify();
            return;
        }
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.refreshing = true;
        let depth = self.revset_depth;
        let prev_change_id = self
            .selected
            .and_then(|ix| self.graph.changes.get(ix))
            .map(|c| c.change_id.clone());

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let entries = repo.log_graph(&build_default_revset(depth)).ok();
                    let bookmarks = repo.list_bookmarks().unwrap_or_default();
                    let workspaces = repo.workspace_list().unwrap_or_default();
                    (entries, bookmarks, workspaces)
                })
                .await;
            let (entries, bookmarks, workspaces) = result;
            let _ = this.update(cx, move |vm, cx| {
                vm.loading.refreshing = false;
                vm.loading.wc_changes = false;
                vm.graph.bookmarks = Arc::new(bookmarks);
                vm.graph.workspaces = Arc::new(workspaces);
                if let Some(entries) = entries {
                    vm.graph.dag_layout = Arc::new(DagLayout::compute(&entries));
                    let changes: Vec<ChangeInfo> =
                        entries.iter().map(|e| e.change.clone()).collect();
                    let new_selected = prev_change_id
                        .as_ref()
                        .and_then(|id| changes.iter().position(|c| &c.change_id == id))
                        .or(changes
                            .iter()
                            .position(|c| c.is_working_copy)
                            .or(if changes.is_empty() { None } else { Some(0) }));
                    vm.graph.changes = Arc::new(changes);
                    vm.graph.entries = Arc::new(entries);
                    // Re-select even if the index is unchanged — file contents may have.
                    if let Some(ix) = new_selected {
                        vm.select_change(ix, cx);
                    } else {
                        vm.selected = None;
                    }
                }
                cx.notify();
            });
        })
        .detach();
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
        cx.spawn(async move |this, cx| {
            let email_for_fetch = email.clone();
            let _ = cx
                .background_spawn(async move {
                    crate::ui::avatar::fetch_blocking(&email_for_fetch);
                })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                vm.avatar_in_flight.remove(&email_for_remove);
                cx.notify();
            });
        })
        .detach();
    }
}

fn compute_diff_blocking(
    repo: &Repo,
    rev: &str,
    hunk: &DiffHunk,
    ignore_whitespace: bool,
) -> Option<FileDiff> {
    let path = hunk.path.clone();
    let (old, new) = match (hunk.old_content.clone(), hunk.new_content.clone()) {
        (Some(o), Some(n)) if !(o.is_empty() && n.is_empty()) => (o, n),
        _ => match repo.show_file(rev, &path) {
            Ok(h) => (
                h.old_content.unwrap_or_default(),
                h.new_content.unwrap_or_default(),
            ),
            Err(_) => return None,
        },
    };
    Some(compute_file_diff(&path, &old, &new, ignore_whitespace))
}
