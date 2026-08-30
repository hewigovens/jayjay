use std::collections::HashSet;
use std::sync::Arc;

use gpui::{App, Context, Modifiers};

use super::RepoWindow;
use crate::ui::ordered_selection::{OrderedSelection, SelectionClick};

#[derive(Default)]
pub(crate) struct FileMultiSelect {
    selection: OrderedSelection<String>,
    /// Change id the selection was made on; a different or fresh (post-split) selected change id voids it.
    change_id: Option<String>,
    /// Cached hunk indices, recomputed only at mutation points so per-frame row rendering just clones the Arc.
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

        let ordered = self.ordered_visible_file_paths(cx);
        let mut selection = std::mem::take(&mut self.file_column.multi_select.selection);
        if selection.is_empty()
            && let Some(primary) = self
                .vm
                .read(cx)
                .selected_hunk()
                .map(|hunk| hunk.path.clone())
        {
            selection.replace(primary);
        }
        selection.apply(SelectionClick::from_modifiers(&modifiers), path, &ordered);
        if let Some(primary_ix) = selection
            .primary()
            .and_then(|primary| self.file_hunk_index(primary, cx))
        {
            self.select_file(primary_ix, cx);
        }
        self.set_file_multi_select(selection, change_id, cx);
        cx.notify();
    }

    pub(crate) fn collapse_file_multi_select(&mut self, hunk_ix: usize, cx: &App) {
        let Some(path) = self.file_path_at(hunk_ix, cx) else {
            return;
        };
        let change_id = self.file_select_change_id(cx);
        if change_id.is_none() {
            self.file_column.multi_select.clear();
            return;
        }
        let mut selection = OrderedSelection::default();
        selection.replace(path);
        self.set_file_multi_select(selection, change_id, cx);
    }

    pub(crate) fn prune_file_multi_select(&mut self, cx: &App) {
        let ms = &self.file_column.multi_select;
        if ms.selection.is_empty() {
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
        ms.selection
            .retain(|path| available.contains(path.as_str()));
        self.refresh_multi_select_hunk_indices(cx);
    }

    pub(crate) fn file_context_selection(&self, clicked: &str, cx: &App) -> Vec<String> {
        let paths = self.multi_selected_file_paths(cx);
        if paths.len() > 1 && paths.iter().any(|p| p == clicked) {
            return paths;
        }
        vec![clicked.to_owned()]
    }

    pub fn multi_selected_file_paths(&self, cx: &App) -> Vec<String> {
        let Some(ms) = self.active_multi_select(cx) else {
            return Vec::new();
        };
        self.ordered_visible_file_paths(cx)
            .into_iter()
            .filter(|path| ms.selection.contains(path))
            .collect()
    }

    pub(crate) fn multi_selected_hunk_indices(&self) -> Arc<HashSet<usize>> {
        self.file_column.multi_select.hunk_indices.clone()
    }

    fn active_multi_select(&self, cx: &App) -> Option<&FileMultiSelect> {
        let ms = &self.file_column.multi_select;
        (!ms.selection.is_empty() && ms.is_valid_for(self.file_select_change_id(cx).as_deref()))
            .then_some(ms)
    }

    fn set_file_multi_select(
        &mut self,
        selection: OrderedSelection<String>,
        change_id: Option<String>,
        cx: &App,
    ) {
        self.file_column.multi_select = FileMultiSelect {
            selection,
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
                        .filter(|(_, hunk)| ms.selection.contains(&hunk.path))
                        .map(|(ix, _)| ix)
                        .collect()
                })
                .unwrap_or_default(),
        };
        self.file_column.multi_select.hunk_indices = Arc::new(indices);
    }

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

    fn file_hunk_index(&self, path: &str, cx: &App) -> Option<usize> {
        self.vm
            .read(cx)
            .files
            .as_ref()
            .and_then(|files| files.iter().position(|hunk| hunk.path == path))
    }

    fn file_path_at(&self, hunk_ix: usize, cx: &App) -> Option<String> {
        self.vm
            .read(cx)
            .files
            .as_ref()
            .and_then(|files| files.get(hunk_ix))
            .map(|hunk| hunk.path.clone())
    }

    fn file_select_change_id(&self, cx: &App) -> Option<String> {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .map(|c| c.change_id.id.clone())
    }
}
