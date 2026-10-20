use gpui::{App, Context};

use super::FileHistoryView;
use crate::repo::window::{SECONDARY_PANE_MAX, SECONDARY_PANE_MIN};
use crate::ui::resizable_pane::ResizablePane;

impl FileHistoryView {
    pub(super) fn list_pane(cx: &App) -> ResizablePane {
        let width = crate::app::config::current(cx).layout.secondary_pane_width;
        ResizablePane::new(
            width,
            SECONDARY_PANE_MIN,
            SECONDARY_PANE_MAX,
            SECONDARY_PANE_MIN,
        )
    }

    pub(super) fn start_list_drag(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        self.list_layout.start_drag(x, viewport_width);
        cx.notify();
    }

    pub(super) fn drag_list_to(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        if self.list_layout.drag_to(x, viewport_width) {
            cx.notify();
        }
    }

    pub(super) fn end_list_drag(&mut self, cx: &mut Context<Self>) {
        if self.list_layout.end_drag().is_some() {
            cx.notify();
        }
    }
}
