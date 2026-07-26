use gpui::{Context, ScrollStrategy};

use super::{ActivePane, RepoWindow};
use crate::ui::navigation::{self, ListNav, ListNavKeys};

impl RepoWindow {
    pub(super) fn handle_nav_key(
        &mut self,
        ev: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.find.query.is_some() {
            return false;
        }
        if self.diff_edit_active() && self.handle_diff_edit_nav_key(ev, cx) {
            return true;
        }
        if let Some(direction) = navigation::list_nav_from_key(ev, ListNavKeys::CONTENT_LIST) {
            self.move_selection(direction, cx);
            return true;
        }

        let m = &ev.keystroke.modifiers;
        if m.platform || m.alt || m.control {
            return false;
        }
        let key = ev.keystroke.key.as_str();

        match key {
            "tab" => {
                self.toggle_pane(cx);
                true
            }
            "space" if matches!(self.active_pane, ActivePane::FileColumn) => {
                self.toggle_reviewed_for_selected_files(cx);
                true
            }
            _ => false,
        }
    }

    fn move_selection(&mut self, direction: ListNav, cx: &mut Context<Self>) {
        match self.active_pane {
            ActivePane::Sidebar => {
                let vm = self.vm.read(cx);
                let len = vm.graph.changes.len();
                if let Some(new) = navigation::move_index(vm.selected, len, direction)
                    && Some(new) != vm.selected
                {
                    self.select_change(new, cx);
                    self.scrolls
                        .changes
                        .scroll_to_item(new, ScrollStrategy::Top);
                }
            }
            ActivePane::FileColumn => self.move_file_selection(direction, cx),
        }
    }

    fn move_file_selection(&mut self, direction: ListNav, cx: &mut Context<Self>) {
        let tree_mode = crate::app::config::current(cx).diff.tree_file_list;
        if tree_mode {
            self.move_file_selection_tree(direction, cx);
            return;
        }
        let (show_review, change_id) = self.review_file_context(cx);
        let vm = self.vm.read(cx);
        let (files, selected_file_ix) = (vm.files.clone(), vm.selected_file_ix);
        let visible = files
            .map(|files| self.visible_indices(&files, change_id.as_deref(), show_review, cx))
            .unwrap_or_default();
        let current = selected_file_ix.and_then(|ix| visible.iter().position(|v| *v == ix));
        if let Some(new_row) = navigation::move_index(current, visible.len(), direction) {
            let new = visible[new_row];
            if Some(new) != selected_file_ix {
                self.select_file(new, cx);
            }
            self.scrolls
                .files
                .scroll_to_item(new_row, ScrollStrategy::Top);
        }
    }

    fn move_file_selection_tree(&mut self, direction: ListNav, cx: &mut Context<Self>) {
        let (show_review, change_id) = self.review_file_context(cx);
        let vm = self.vm.read(cx);
        let Some(hunks) = vm.files.clone() else {
            return;
        };
        let selected_hunk = vm.selected_file_ix;
        let visible_indices = self.visible_indices(&hunks, change_id.as_deref(), show_review, cx);
        if visible_indices.is_empty() {
            return;
        }
        let visible_indices = std::sync::Arc::new(visible_indices);
        let tree = self.file_tree_cache.borrow_mut().visible(
            &hunks,
            &visible_indices,
            &self.collapsed_dirs,
        );
        let selected_visible_hunk =
            selected_hunk.and_then(|ix| visible_indices.iter().position(|v| *v == ix));
        let Some((row, visible_hunk)) = next_tree_file(&tree, selected_visible_hunk, direction)
        else {
            return;
        };
        let Some(hunk) = visible_indices.get(visible_hunk).copied() else {
            return;
        };
        if Some(hunk) != selected_hunk {
            self.select_file(hunk, cx);
        }
        self.scrolls.tree_files.scroll_to_top_of_item(row);
    }

    fn toggle_pane(&mut self, cx: &mut Context<Self>) {
        self.active_pane = match self.active_pane {
            ActivePane::Sidebar => ActivePane::FileColumn,
            ActivePane::FileColumn => ActivePane::Sidebar,
        };
        cx.notify();
    }
}

/// Returns `(visible row index, hunk index)` — the row differs from the hunk index when a directory row precedes the file. A hidden selection lands on the first file.
fn next_tree_file(
    tree: &[jayjay_core::FileTreeEntry],
    selected_hunk: Option<usize>,
    direction: ListNav,
) -> Option<(usize, usize)> {
    let files: Vec<(usize, usize)> = tree
        .iter()
        .enumerate()
        .filter_map(|(row, e)| e.hunk_index.map(|h| (row, h as usize)))
        .collect();
    if files.is_empty() {
        return None;
    }
    let current = selected_hunk.and_then(|h| files.iter().position(|(_, hunk)| *hunk == h));
    let next = match current {
        Some(pos) => navigation::move_index(Some(pos), files.len(), direction)?,
        None => 0,
    };
    Some(files[next])
}

#[cfg(test)]
mod tests {
    use super::*;
    use jayjay_core::FileTreeEntry;

    fn dir(path: &str) -> FileTreeEntry {
        FileTreeEntry {
            name: path.into(),
            path: path.into(),
            depth: 0,
            hunk_index: None,
        }
    }

    fn file(path: &str, hunk: u32) -> FileTreeEntry {
        FileTreeEntry {
            name: path.into(),
            path: path.into(),
            depth: 1,
            hunk_index: Some(hunk),
        }
    }

    fn sample() -> Vec<FileTreeEntry> {
        vec![
            dir("src"),
            file("a.rs", 0),
            file("b.rs", 1),
            file("z.txt", 2),
        ]
    }

    #[test]
    fn next_returns_visible_row_not_hunk_index() {
        assert_eq!(
            next_tree_file(&sample(), Some(0), ListNav::Next),
            Some((2, 1))
        );
    }

    #[test]
    fn previous_steps_back_over_visible_files() {
        assert_eq!(
            next_tree_file(&sample(), Some(2), ListNav::Previous),
            Some((2, 1))
        );
    }

    #[test]
    fn skips_files_hidden_under_collapsed_dir() {
        let tree = vec![dir("src"), file("z.txt", 2)];
        assert_eq!(next_tree_file(&tree, Some(0), ListNav::Next), Some((1, 2)));
        assert_eq!(next_tree_file(&tree, Some(2), ListNav::Next), Some((1, 2)));
    }

    #[test]
    fn no_files_yields_none() {
        assert_eq!(next_tree_file(&[dir("src")], Some(0), ListNav::Next), None);
    }
}
