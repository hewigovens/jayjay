use std::sync::Arc;

use gpui::{AppContext, Context};
use jayjay_core::dag::DagLayout;
use jayjay_core::diff::{FileDiff, compute_file_diff};
use jayjay_core::{
    BookmarkInfo, ChangeInfo, CoreResult, DiffHunk, GraphEntry, Repo, WorkspaceInfo,
    build_default_revset,
};

use super::RepoViewModel;
use crate::diff::DetailMode;
use crate::repo::revset;

impl RepoViewModel {
    pub(in crate::repo) fn load_diff_async(
        &mut self,
        rev: String,
        hunk: DiffHunk,
        cx: &mut Context<Self>,
    ) {
        let compare_from_rev = self
            .compare
            .as_ref()
            .map(|compare| compare.from_rev.clone());
        let cache_key = diff_cache_key(
            compare_from_rev.as_deref(),
            &rev,
            &hunk,
            self.ignore_whitespace,
        );
        if let Some(cached) = self.diff_cache.get(&cache_key).cloned() {
            self.current_diff = cached;
            self.loading.diff = false;
            if matches!(self.detail_mode, DetailMode::Annotate) {
                self.load_annotate(cx);
            }
            cx.notify();
            return;
        }

        self.loading.diff_gen = self.loading.diff_gen.wrapping_add(1);
        let generation = self.loading.diff_gen;
        self.current_diff = None;
        self.loading.diff = true;

        let Some(repo) = self.repo.clone() else {
            self.loading.diff = false;
            cx.notify();
            return;
        };
        let fallback_path = hunk.path.clone();
        let ignore_whitespace = self.ignore_whitespace;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let file_diff = cx
                .background_spawn(async move {
                    compute_diff_blocking(
                        &repo,
                        &rev,
                        &hunk,
                        compare_from_rev.as_deref(),
                        ignore_whitespace,
                    )
                })
                .await;

            let _ = this.update(cx, move |vm, cx| {
                if vm.loading.diff_gen != generation {
                    return;
                }
                vm.loading.diff = false;
                match file_diff {
                    Ok(file_diff) => {
                        let file_diff = Arc::new(file_diff);
                        vm.diff_cache.insert(cache_key, Some(file_diff.clone()));
                        vm.current_diff = Some(file_diff);
                    }
                    Err(error) => {
                        vm.current_diff = Some(Arc::new(FileDiff {
                            path: fallback_path,
                            language: String::new(),
                            lines: Vec::new(),
                            whitespace_only_hidden: false,
                        }));
                        vm.present_error(error);
                    }
                }
                if matches!(vm.detail_mode, DetailMode::Annotate) {
                    vm.load_annotate(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::repo) fn preload_diffs_async(
        &mut self,
        hunks: Arc<Vec<DiffHunk>>,
        cx: &mut Context<Self>,
    ) {
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
        let ignore_whitespace = self.ignore_whitespace;
        let pending: Vec<_> = hunks
            .iter()
            .enumerate()
            .filter(|(ix, _)| Some(*ix) != self.selected_file_ix)
            .map(|hunk| {
                let hunk = hunk.1;
                (
                    diff_cache_key(None, &rev, hunk, ignore_whitespace),
                    hunk.clone(),
                )
            })
            .filter(|(key, _)| !self.diff_cache.contains_key(key))
            .collect();

        if pending.is_empty() {
            return;
        }

        for (cache_key, hunk) in pending {
            let repo = repo.clone();
            let rev = rev.clone();
            cx.spawn(async move |this, cx| {
                let Ok(file_diff) = cx
                    .background_spawn(async move {
                        compute_diff_blocking(&repo, &rev, &hunk, None, ignore_whitespace)
                    })
                    .await
                else {
                    return;
                };
                let file_diff = Arc::new(file_diff);

                let _ = this.update(cx, move |vm, _cx| {
                    vm.diff_cache.entry(cache_key).or_insert(Some(file_diff));
                });
            })
            .detach();
        }
    }

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
        self.clear_error();
        let depth = self.revset_depth;
        let previous_selection = self
            .selected
            .and_then(|ix| self.graph.changes.get(ix))
            .map(|c| (c.change_id.clone(), c.commit_id.clone()));

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { refresh_graph_blocking(&repo, depth) })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                vm.loading.refreshing = false;
                vm.loading.wc_changes = false;
                match result {
                    Ok(data) => {
                        let entries = data.entries;
                        vm.graph.bookmarks = Arc::new(data.bookmarks);
                        vm.graph.workspaces = Arc::new(data.workspaces);
                        vm.graph.dag_layout = Arc::new(DagLayout::compute(&entries));
                        let changes: Vec<ChangeInfo> =
                            entries.iter().map(|e| e.change.clone()).collect();
                        let new_selected = previous_selection
                            .as_ref()
                            .and_then(|(_, commit_id)| {
                                changes.iter().position(|c| &c.commit_id == commit_id)
                            })
                            .or_else(|| {
                                previous_selection.as_ref().and_then(|(change_id, _)| {
                                    changes.iter().position(|c| &c.change_id == change_id)
                                })
                            })
                            .or_else(|| changes.iter().position(|c| c.is_working_copy))
                            .or(if changes.is_empty() { None } else { Some(0) });
                        vm.graph.changes = Arc::new(changes);
                        vm.graph.entries = Arc::new(entries);
                        // Re-select even if the index is unchanged — file contents may have.
                        if let Some(ix) = new_selected {
                            vm.select_change(ix, cx);
                        } else {
                            vm.selected = None;
                        }
                    }
                    Err(error) => vm.present_error(error),
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

struct RefreshData {
    entries: Vec<GraphEntry>,
    bookmarks: Vec<BookmarkInfo>,
    workspaces: Vec<WorkspaceInfo>,
}

fn refresh_graph_blocking(repo: &Repo, depth: u32) -> CoreResult<RefreshData> {
    repo.refresh_working_copy()?;
    let entries = repo.log_graph(&build_default_revset(depth))?;
    let bookmarks = repo.list_bookmarks().unwrap_or_default();
    let workspaces = repo.workspace_list().unwrap_or_default();
    Ok(RefreshData {
        entries,
        bookmarks,
        workspaces,
    })
}

fn compute_diff_blocking(
    repo: &Repo,
    rev: &str,
    hunk: &DiffHunk,
    compare_from_rev: Option<&str>,
    ignore_whitespace: bool,
) -> CoreResult<FileDiff> {
    let path = hunk.path.clone();
    let (old, new) = match (hunk.old_content.clone(), hunk.new_content.clone()) {
        (Some(o), Some(n)) if !(o.is_empty() && n.is_empty()) => (o, n),
        _ => {
            let h = if let Some(from_rev) = compare_from_rev {
                repo.interdiff_file(from_rev, rev, &path)
            } else {
                repo.show_file(rev, &path)
            };
            let h = h?;
            (
                h.old_content.unwrap_or_default(),
                h.new_content.unwrap_or_default(),
            )
        }
    };
    Ok(compute_file_diff(&path, &old, &new, ignore_whitespace))
}

fn diff_cache_key(
    compare_from_rev: Option<&str>,
    rev: &str,
    hunk: &DiffHunk,
    ignore_whitespace: bool,
) -> String {
    format!(
        "{}\0{}\0{}\0{}\0{}",
        compare_from_rev.unwrap_or(""),
        rev,
        hunk.path,
        hunk.review_identity,
        ignore_whitespace
    )
}
