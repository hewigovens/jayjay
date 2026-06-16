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
                self.toggle_reviewed_for_selected_file(cx);
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

    /// Move the file-column selection by one step. Flat mode steps the hunk index directly;
    /// tree mode delegates so it can skip hidden files and scroll by tree-row index.
    fn move_file_selection(&mut self, direction: ListNav, cx: &mut Context<Self>) {
        let tree_mode = crate::app::config::current(cx).diff.tree_file_list;
        if tree_mode {
            self.move_file_selection_tree(direction, cx);
            return;
        }
        let vm = self.vm.read(cx);
        let len = vm.files.as_ref().map(|f| f.len()).unwrap_or(0);
        if let Some(new) = navigation::move_index(vm.selected_file_ix, len, direction)
            && Some(new) != vm.selected_file_ix
        {
            self.select_file(new, cx);
            self.scrolls.files.scroll_to_item(new, ScrollStrategy::Top);
        }
    }

    fn move_file_selection_tree(&mut self, direction: ListNav, cx: &mut Context<Self>) {
        let Some(hunks) = self.vm.read(cx).files.clone() else {
            return;
        };
        let tree = self
            .file_tree_cache
            .borrow_mut()
            .visible(&hunks, &self.collapsed_dirs);
        let selected_hunk = self.vm.read(cx).selected_file_ix;
        let Some((row, hunk)) = next_tree_file(&tree, selected_hunk, direction) else {
            return;
        };
        if Some(hunk) != selected_hunk {
            self.select_file(hunk, cx);
        }
        self.scrolls.files.scroll_to_item(row, ScrollStrategy::Top);
    }

    fn toggle_pane(&mut self, cx: &mut Context<Self>) {
        self.active_pane = match self.active_pane {
            ActivePane::Sidebar => ActivePane::FileColumn,
            ActivePane::FileColumn => ActivePane::Sidebar,
        };
        cx.notify();
    }
}

/// Resolve the next file selection in tree mode, returning `(visible row index, hunk index)`.
/// Steps only over visible file rows so it skips files under collapsed dirs; the row differs
/// from the hunk index when a directory row precedes the file. Hidden selection lands on the first file.
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

    // dir "src" (row 0), src/a.rs hunk 0 (row 1), src/b.rs hunk 1 (row 2),
    // root z.txt hunk 2 (row 3). Row indices differ from hunk indices.
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
        // From hunk 0 (row 1) → hunk 1 at row 2, not row 1.
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
        // src collapsed: only z.txt (hunk 2) is visible at row 1.
        let tree = vec![dir("src"), file("z.txt", 2)];
        // Selection sits on hidden hunk 0 → land on the first visible file.
        assert_eq!(next_tree_file(&tree, Some(0), ListNav::Next), Some((1, 2)));
        // Stepping from the visible file stays on it (no hidden neighbor).
        assert_eq!(next_tree_file(&tree, Some(2), ListNav::Next), Some((1, 2)));
    }

    #[test]
    fn no_files_yields_none() {
        assert_eq!(next_tree_file(&[dir("src")], Some(0), ListNav::Next), None);
    }
}
