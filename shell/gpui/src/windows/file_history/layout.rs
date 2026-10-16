use gpui::{App, Context};

use super::FileHistoryView;
use crate::repo::window::{SECONDARY_PANE_MAX, SECONDARY_PANE_MIN};
use crate::ui::pane_drag::{PaneDrag, pane_max};
use crate::ui::resize_handle::RESIZE_HANDLE_WIDTH;

pub(super) struct HistoryListLayout {
    width: f32,
    drag: Option<PaneDrag<()>>,
}

impl HistoryListLayout {
    pub(super) fn from_config(cx: &App) -> Self {
        let width = crate::app::config::current(cx).layout.secondary_pane_width;
        Self {
            width: width.clamp(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX),
            drag: None,
        }
    }

    pub(super) fn fitted(&self, viewport_width: f32) -> f32 {
        self.width.min(Self::max(viewport_width))
    }

    fn max(viewport_width: f32) -> f32 {
        let room = viewport_width - RESIZE_HANDLE_WIDTH - SECONDARY_PANE_MIN;
        pane_max(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX, room)
    }
}

impl FileHistoryView {
    pub(super) fn start_list_drag(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        let width = self.list_layout.fitted(viewport_width);
        self.list_layout.drag = Some(PaneDrag::new((), x, width));
        cx.notify();
    }

    pub(super) fn drag_list_to(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.list_layout.drag else {
            return;
        };
        self.list_layout.width = drag.width_at(
            x,
            SECONDARY_PANE_MIN,
            HistoryListLayout::max(viewport_width),
        );
        cx.notify();
    }

    pub(super) fn end_list_drag(&mut self, cx: &mut Context<Self>) {
        if self.list_layout.drag.take().is_some() {
            cx.notify();
        }
    }
}
