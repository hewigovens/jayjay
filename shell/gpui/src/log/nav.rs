use gpui::{Context, ScrollStrategy};

use super::{ActivePane, LogView};
use crate::ui::navigation::{self, ListNav, ListNavKeys};

impl LogView {
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
            ActivePane::FileColumn => {
                let vm = self.vm.read(cx);
                let len = vm.files.as_ref().map(|f| f.len()).unwrap_or(0);
                if let Some(new) = navigation::move_index(vm.selected_file_ix, len, direction)
                    && Some(new) != vm.selected_file_ix
                {
                    self.select_file(new, cx);
                    self.scrolls.files.scroll_to_item(new, ScrollStrategy::Top);
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
