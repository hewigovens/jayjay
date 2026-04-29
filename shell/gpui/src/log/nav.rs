use gpui::{Context, ScrollStrategy};

use super::{ActivePane, LogView};

impl LogView {
    pub(super) fn handle_nav_key(
        &mut self,
        ev: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.find_query.is_some() {
            return false;
        }
        let m = &ev.keystroke.modifiers;
        if m.platform || m.control || m.alt {
            return false;
        }
        let key = ev.keystroke.key.as_str();
        let delta: i32 = match key {
            "j" | "down" => 1,
            "k" | "up" => -1,
            "tab" => {
                self.toggle_pane(cx);
                return true;
            }
            "space" if matches!(self.active_pane, ActivePane::FileColumn) => {
                self.toggle_reviewed_for_selected_file(cx);
                return true;
            }
            _ => return false,
        };
        self.move_selection(delta, cx);
        true
    }

    fn move_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        match self.active_pane {
            ActivePane::Sidebar => {
                let vm = self.vm.read(cx);
                let len = vm.graph.changes.len();
                if len == 0 {
                    return;
                }
                let cur = vm.selected.unwrap_or(0) as i32;
                let new = (cur + delta).clamp(0, len as i32 - 1) as usize;
                if Some(new) != vm.selected {
                    self.select_change(new, cx);
                    self.changes_scroll.scroll_to_item(new, ScrollStrategy::Top);
                }
            }
            ActivePane::FileColumn => {
                let vm = self.vm.read(cx);
                let len = vm.files.as_ref().map(|f| f.len()).unwrap_or(0);
                if len == 0 {
                    return;
                }
                let cur = vm.selected_file_ix.unwrap_or(0) as i32;
                let new = (cur + delta).clamp(0, len as i32 - 1) as usize;
                if Some(new) != vm.selected_file_ix {
                    self.select_file(new, cx);
                    self.files_scroll.scroll_to_item(new, ScrollStrategy::Top);
                }
            }
        }
    }

    fn toggle_pane(&mut self, cx: &mut Context<Self>) {
        self.active_pane = match self.active_pane {
            ActivePane::Sidebar => ActivePane::FileColumn,
            ActivePane::FileColumn => ActivePane::Sidebar,
        };
        cx.notify();
    }
}
