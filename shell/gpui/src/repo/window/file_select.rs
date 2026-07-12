//! File-column multi-select, mirroring the SwiftUI shell's model: plain click selects one file, shift-click extends a range from the anchor, and the platform secondary modifier (cmd on macOS, ctrl elsewhere) toggles single files in and out.

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{App, Context, Modifiers};

use super::RepoWindow;

/// Paths multi-selected in the file column, feeding batch context-menu actions and row highlighting; the primary selection (which diff is shown) stays `vm.selected_file_ix`.
#[derive(Default)]
pub(crate) struct FileMultiSelect {
    paths: HashSet<String>,
    anchor: Option<String>,
    /// Change id the selection was made on; a different or fresh (post-split) selected change id voids it.
    change_id: Option<String>,
    /// Cached hunk indices of `paths`, recomputed only at mutation points so per-frame row rendering just clones the Arc.
    hunk_indices: Arc<HashSet<usize>>,
}

impl FileMultiSelect {
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    fn is_valid_for(&self, change_id: Option<&str>) -> bool {
        change_id.is_some() && self.change_id.as_deref() == change_id
    }
}

impl RepoWindow {
    /// `pub` so the separate `tests/` crate can drive selection transitions without synthesizing real mouse events.
    pub fn handle_file_row_click(
        &mut self,
        hunk_ix: usize,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.file_path_at(hunk_ix, cx) else {
            return;
        };
        let change_id = self.file_select_change_id(cx);
        if !self
            .file_column
            .multi_select
            .is_valid_for(change_id.as_deref())
        {
            self.file_column.multi_select.clear();
        }

        if modifiers.shift
            && let Some((paths, anchor)) = self.shift_range_paths(&path, cx)
        {
            // `select_file` collapses the set to the clicked file, so the range overwrite must come after it.
            self.select_file(hunk_ix, cx);
            self.set_file_multi_select(paths, anchor, change_id, cx);
            cx.notify();
            return;
        }

        if modifiers.secondary() {
            let mut paths = std::mem::take(&mut self.file_column.multi_select.paths);
            if !paths.remove(&path) {
                paths.insert(path.clone());
                self.select_file(hunk_ix, cx);
            }
            self.set_file_multi_select(paths, Some(path), change_id, cx);
            cx.notify();
            return;
        }

        self.select_file(hunk_ix, cx);
    }

    /// Every single selection (click, keyboard nav, filters) collapses the multi-selection to that file, matching SwiftUI's `selectSingleFile`.
    pub(crate) fn collapse_file_multi_select(&mut self, hunk_ix: usize, cx: &App) {
        let Some(path) = self.file_path_at(hunk_ix, cx) else {
            return;
        };
        let change_id = self.file_select_change_id(cx);
        if change_id.is_none() {
            self.file_column.multi_select.clear();
            return;
        }
        self.set_file_multi_select(HashSet::from([path.clone()]), Some(path), change_id, cx);
    }

    /// Runs on every view-model change: drops the selection when the change switched (or compare started) and intersects it with the reloaded file list, mirroring SwiftUI's `restoreFileSelection`.
    pub(crate) fn prune_file_multi_select(&mut self, cx: &App) {
        let ms = &self.file_column.multi_select;
        if ms.paths.is_empty() && ms.anchor.is_none() {
            return;
        }
        if !ms.is_valid_for(self.file_select_change_id(cx).as_deref()) {
            self.file_column.multi_select.clear();
            return;
        }
        // Files still loading: the observer runs again when they land.
        let Some(files) = self.vm.read(cx).files.clone() else {
            return;
        };
        let available: HashSet<&str> = files.iter().map(|h| h.path.as_str()).collect();
        let ms = &mut self.file_column.multi_select;
        ms.paths.retain(|p| available.contains(p.as_str()));
        if ms.anchor.as_deref().is_some_and(|a| !available.contains(a)) {
            ms.anchor = None;
        }
        self.refresh_multi_select_hunk_indices(cx);
    }

    /// SwiftUI parity (`contextSelectionPaths`): a right-click inside a >1 selection targets the whole selection in visible order; anywhere else targets just the clicked file.
    pub(crate) fn file_context_selection(&self, clicked: &str, cx: &App) -> Vec<String> {
        let paths = self.multi_selected_file_paths(cx);
        if paths.len() > 1 && paths.iter().any(|p| p == clicked) {
            return paths;
        }
        vec![clicked.to_owned()]
    }

    /// Multi-selected paths in visible order; `pub` so the separate `tests/` crate can assert selection transitions.
    pub fn multi_selected_file_paths(&self, cx: &App) -> Vec<String> {
        let Some(ms) = self.active_multi_select(cx) else {
            return Vec::new();
        };
        self.ordered_visible_file_paths(cx)
            .into_iter()
            .filter(|p| ms.paths.contains(p))
            .collect()
    }

    /// Hunk indices highlighted as part of the multi-selection; cached, so the per-frame render path only clones the Arc.
    pub(crate) fn multi_selected_hunk_indices(&self) -> Arc<HashSet<usize>> {
        self.file_column.multi_select.hunk_indices.clone()
    }

    /// The multi-selection when it is non-empty and still valid for the selected change; the shared guard for every consumer.
    fn active_multi_select(&self, cx: &App) -> Option<&FileMultiSelect> {
        let ms = &self.file_column.multi_select;
        (!ms.paths.is_empty() && ms.is_valid_for(self.file_select_change_id(cx).as_deref()))
            .then_some(ms)
    }

    fn set_file_multi_select(
        &mut self,
        paths: HashSet<String>,
        anchor: Option<String>,
        change_id: Option<String>,
        cx: &App,
    ) {
        self.file_column.multi_select = FileMultiSelect {
            paths,
            anchor,
            change_id,
            hunk_indices: Arc::default(),
        };
        self.refresh_multi_select_hunk_indices(cx);
    }

    fn refresh_multi_select_hunk_indices(&mut self, cx: &App) {
        let indices = match self.active_multi_select(cx) {
            None => HashSet::new(),
            Some(ms) => self
                .vm
                .read(cx)
                .files
                .as_ref()
                .map(|files| {
                    files
                        .iter()
                        .enumerate()
                        .filter(|(_, hunk)| ms.paths.contains(&hunk.path))
                        .map(|(ix, _)| ix)
                        .collect()
                })
                .unwrap_or_default(),
        };
        self.file_column.multi_select.hunk_indices = Arc::new(indices);
    }

    /// Selectable paths in on-screen order — filtered flat order, or tree traversal order minus collapsed dirs — mirroring SwiftUI's `visibleSelectablePaths`.
    fn ordered_visible_file_paths(&self, cx: &App) -> Vec<String> {
        if !crate::app::config::current(cx).diff.tree_file_list {
            return self.visible_file_paths(cx);
        }
        let (show_review, change_id) = self.review_file_context(cx);
        let Some(files) = self.vm.read(cx).files.clone() else {
            return Vec::new();
        };
        let visible = Arc::new(self.visible_indices(&files, change_id.as_deref(), show_review, cx));
        let tree =
            self.file_tree_cache
                .borrow_mut()
                .visible(&files, &visible, &self.collapsed_dirs);
        tree.iter()
            .filter_map(|entry| entry.hunk_index)
            .filter_map(|vix| visible.get(vix as usize).copied())
            .filter_map(|ix| files.get(ix).map(|h| h.path.clone()))
            .collect()
    }

    /// Range endpoints resolve against the visible order; a missing anchor falls back to the primary file (SwiftUI seeds the anchor with the auto-selected file).
    fn shift_range_paths(
        &self,
        clicked: &str,
        cx: &App,
    ) -> Option<(HashSet<String>, Option<String>)> {
        let anchor = self
            .file_column
            .multi_select
            .anchor
            .clone()
            .or_else(|| self.vm.read(cx).selected_hunk().map(|h| h.path.clone()))?;
        let ordered = self.ordered_visible_file_paths(cx);
        let a = ordered.iter().position(|p| p == &anchor)?;
        let b = ordered.iter().position(|p| p == clicked)?;
        let (lo, hi) = (a.min(b), a.max(b));
        Some((ordered[lo..=hi].iter().cloned().collect(), Some(anchor)))
    }

    fn file_path_at(&self, hunk_ix: usize, cx: &App) -> Option<String> {
        self.vm
            .read(cx)
            .files
            .as_ref()
            .and_then(|files| files.get(hunk_ix))
            .map(|hunk| hunk.path.clone())
    }

    /// `None` in compare mode: the displayed interdiff's files are not the selected change's files, so no multi-selection applies there.
    fn file_select_change_id(&self, cx: &App) -> Option<String> {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .map(|c| c.change_id.id.clone())
    }
}
