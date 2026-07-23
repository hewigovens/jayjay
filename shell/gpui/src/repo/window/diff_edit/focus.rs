use gpui::{Context, KeyDownEvent, ScrollStrategy};

use super::rows::DiffEditRow;
use crate::repo::window::RepoWindow;
use crate::ui::navigation::{self, ListNav, ListNavKeys};

impl RepoWindow {
    pub(crate) fn diff_edit_take_pending_focus(&mut self) -> bool {
        let pending = self.diff_edit.active && self.diff_edit.focus_pending;
        if pending {
            self.diff_edit.focus_pending = false;
        }
        pending
    }

    pub fn diff_edit_focused(&self) -> Option<String> {
        self.diff_edit.focused.clone()
    }

    pub(super) fn diff_edit_is_focused(&self, path: &str) -> bool {
        self.diff_edit.focused.as_deref() == Some(path)
    }

    pub(crate) fn handle_diff_edit_nav_key(
        &mut self,
        ev: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if ev.keystroke.key == "enter" && !ev.keystroke.modifiers.modified() {
            let Some(path) = self.diff_edit.focused.clone() else {
                return false;
            };
            self.toggle_diff_edit_collapse(&path, cx);
            self.scroll_diff_edit_focus_into_view(cx);
            return true;
        }
        if !ev.keystroke.modifiers.modified() {
            match ev.keystroke.key.as_str() {
                "left" => return self.set_focused_diff_edit_collapsed(true, cx),
                "right" => return self.set_focused_diff_edit_collapsed(false, cx),
                "space" => {
                    // Consumed even unfocused; falling through would toggle the hidden file column's review mark.
                    if let Some(path) = self.diff_edit.focused.clone() {
                        self.toggle_diff_edit_file(&path, cx);
                    }
                    return true;
                }
                _ => {}
            }
        }
        if let Some(direction) = navigation::list_nav_from_key(ev, ListNavKeys::CONTENT_LIST) {
            self.move_diff_edit_focus(direction, cx);
            return true;
        }
        false
    }

    fn set_focused_diff_edit_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) -> bool {
        let Some(path) = self.diff_edit.focused.clone() else {
            return false;
        };
        if self.diff_edit.collapsed.contains(&path) == collapsed {
            return false;
        }
        self.toggle_diff_edit_collapse(&path, cx);
        self.scroll_diff_edit_focus_into_view(cx);
        true
    }

    fn move_diff_edit_focus(&mut self, direction: ListNav, cx: &mut Context<Self>) {
        let model = self.diff_edit_row_model(cx);
        let len = model.files.len();
        if len == 0 {
            return;
        }
        let current = self
            .diff_edit
            .focused
            .as_deref()
            .and_then(|path| model.file_index(path));
        let next = match current {
            Some(pos) => navigation::move_index(Some(pos), len, direction),
            None => Some(match direction {
                ListNav::Next => 0,
                ListNav::Previous => len - 1,
            }),
        };
        let Some(next) = next else {
            return;
        };
        let path = model.files[next].path.to_string();
        if self.diff_edit.focused.as_deref() != Some(path.as_str()) {
            self.diff_edit.focused = Some(path);
            cx.notify();
        }
        self.scroll_diff_edit_focus_into_view(cx);
    }

    pub fn focus_and_toggle_diff_edit_collapse(&mut self, path: &str, cx: &mut Context<Self>) {
        self.diff_edit.focused = Some(path.to_owned());
        self.toggle_diff_edit_collapse(path, cx);
    }

    fn scroll_diff_edit_focus_into_view(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.diff_edit.focused.clone() else {
            return;
        };
        let model = self.diff_edit_row_model(cx);
        let Some(file_ix) = model.file_index(&path) else {
            return;
        };
        let Some(row) = model
            .rows
            .iter()
            .position(|row| matches!(row, DiffEditRow::Header(ix) if *ix == file_ix))
        else {
            return;
        };
        self.diff_edit
            .scroll
            .scroll_to_item(row, ScrollStrategy::Top);
    }
}
