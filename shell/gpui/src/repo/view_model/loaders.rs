use std::sync::Arc;
use std::time::Duration;

use gpui::{Context, SharedString};
use jayjay_core::dag::DagLayout;
use jayjay_core::diff::{FileDiff, compute_file_diff};
use jayjay_core::{
    BookmarkInfo, ChangeInfo, CoreResult, DiffHunk, DiffPreview, DiffStats, GraphEntry, Repo,
    WorkspaceInfo, build_default_revset,
};

use super::RepoViewModel;
use crate::diff::DetailMode;
use crate::repo::revset;

/// Window during which FS echoes from our own mutations are ignored.
const MUTATION_ECHO_WINDOW: Duration = Duration::from_secs(5);

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

        Self::background_update(
            cx,
            async move {
                compute_diff_blocking(
                    &repo,
                    &rev,
                    &hunk,
                    compare_from_rev.as_deref(),
                    ignore_whitespace,
                )
            },
            move |vm, file_diff, cx| {
                if vm.loading.diff_gen != generation {
                    return;
                }
                vm.loading.diff = false;
                match file_diff {
                    Ok((file_diff, old_preview, new_preview)) => {
                        let file_diff = Arc::new(file_diff);
                        vm.diff_cache.insert(cache_key, Some(file_diff.clone()));
                        vm.current_diff = Some(file_diff);
                        vm.apply_hunk_previews(&fallback_path, old_preview, new_preview);
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
            },
        );
    }

    /// Attach per-file image previews onto the matching file-list hunk so the
    /// diff view can detect and render image diffs. Cheap: only image files carry
    /// previews, and `files` persists within a change view (`diff_cache` is
    /// cleared whenever a new change is selected).
    fn apply_hunk_previews(
        &mut self,
        path: &str,
        old_preview: Option<DiffPreview>,
        new_preview: Option<DiffPreview>,
    ) {
        if old_preview.is_none() && new_preview.is_none() {
            return;
        }
        if let Some(files) = self.files.as_mut()
            && let Some(h) = Arc::make_mut(files).iter_mut().find(|h| h.path == path)
        {
            h.old.preview = old_preview;
            h.new.preview = new_preview;
        }
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
            .map(|(_, hunk)| {
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
            let hunk_path = hunk.path.clone();
            Self::background_update(
                cx,
                async move { compute_diff_blocking(&repo, &rev, &hunk, None, ignore_whitespace) },
                move |vm, result, _cx| {
                    let Ok((file_diff, old_preview, new_preview)) = result else {
                        return;
                    };
                    vm.diff_cache
                        .entry(cache_key)
                        .or_insert(Some(Arc::new(file_diff)));
                    vm.apply_hunk_previews(&hunk_path, old_preview, new_preview);
                },
            );
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

    /// FS-watcher entry point.
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

/// Returns the text diff plus the image previews for the file. The fast file
/// list (`show_summary` → `diff_file_list`) omits previews, so we pull them from
/// the per-file `show_file`; the diff view needs them to render image diffs.
fn compute_diff_blocking(
    repo: &Repo,
    rev: &str,
    hunk: &DiffHunk,
    compare_from_rev: Option<&str>,
    ignore_whitespace: bool,
) -> CoreResult<(FileDiff, Option<DiffPreview>, Option<DiffPreview>)> {
    let path = hunk.path.clone();
    // A byte-identical rename has nothing to diff; loading by the new path alone would render every line as added.
    if hunk.is_content_free_rename() {
        return Ok((
            compute_file_diff(&path, "", "", ignore_whitespace),
            None,
            None,
        ));
    }
    let mut old_preview = hunk.old.preview.clone();
    let mut new_preview = hunk.new.preview.clone();
    let mut projection = hunk.projection.clone();
    let (old, new) = match (hunk.old.content.clone(), hunk.new.content.clone()) {
        (Some(o), Some(n)) if !(o.is_empty() && n.is_empty()) => (o, n),
        _ => {
            let h = if let Some(from_rev) = compare_from_rev {
                repo.interdiff_file(from_rev, rev, &path)
            } else {
                repo.show_file(rev, &path)
            };
            let h = h?;
            old_preview = h.old.preview.clone();
            new_preview = h.new.preview.clone();
            projection = h.projection.clone();
            (
                h.old.content.unwrap_or_default(),
                h.new.content.unwrap_or_default(),
            )
        }
    };
    let diff_path = projection
        .as_ref()
        .map(|projection| projection.virtual_path.as_str())
        .unwrap_or(&path);
    Ok((
        compute_file_diff(diff_path, &old, &new, ignore_whitespace),
        old_preview,
        new_preview,
    ))
}

fn diff_cache_key(
    compare_from_rev: Option<&str>,
    rev: &str,
    hunk: &DiffHunk,
    ignore_whitespace: bool,
) -> String {
    format!(
        "{}\0{}\0{}\0{}\0{}\0{}",
        compare_from_rev.unwrap_or(""),
        rev,
        hunk.path,
        hunk.review_identity,
        hunk.projection
            .as_ref()
            .map(|projection| projection.identity_part())
            .unwrap_or_else(|| "raw".to_owned()),
        ignore_whitespace
    )
}
