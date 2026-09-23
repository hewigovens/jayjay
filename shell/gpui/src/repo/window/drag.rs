use gpui::Context;

use super::{
    ColumnDrag, DragTarget, LayoutState, PREVIEW_MIN, RepoWindow, SECONDARY_PANE_MAX,
    SECONDARY_PANE_MIN, SIDEBAR_MAX, SIDEBAR_MIN, pane_max,
};
use crate::ui::resize_handle::RESIZE_HANDLE_WIDTH;

impl LayoutState {
    fn sidebar_max(viewport_width: f32) -> f32 {
        let room = viewport_width - SECONDARY_PANE_MIN - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        pane_max(SIDEBAR_MIN, SIDEBAR_MAX, room)
    }

    pub(crate) fn sidebar_pane_width(&self, viewport_width: f32) -> f32 {
        self.sidebar_width.min(Self::sidebar_max(viewport_width))
    }

    fn file_column_max(viewport_width: f32, sidebar_width: f32) -> f32 {
        let room = viewport_width - sidebar_width - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        pane_max(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX, room)
    }

    pub(crate) fn fitted(&self, viewport_width: f32) -> (f32, f32) {
        let sidebar = if self.sidebar_hidden && !self.sidebar_closing {
            0.
        } else {
            self.sidebar_pane_width(viewport_width)
        };
        let file_column = self
            .file_column_width
            .min(Self::file_column_max(viewport_width, sidebar));
        (sidebar, file_column)
    }
}

impl RepoWindow {
    pub(crate) fn start_drag(
        &mut self,
        target: DragTarget,
        start_pos: f32,
        viewport_width: f32,
        cx: &mut Context<Self>,
    ) {
        let (sidebar_width, file_column_width) = self.layout.fitted(viewport_width);
        let start_size = match target {
            DragTarget::Sidebar => sidebar_width,
            DragTarget::FileColumn => file_column_width,
        };
        self.layout.drag = Some(ColumnDrag {
            target,
            start_pos,
            start_size,
        });
        cx.notify();
    }

    pub(crate) fn drag_to(&mut self, current_x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.layout.drag else {
            return;
        };
        match drag.target {
            DragTarget::Sidebar => {
                let new_size = drag.start_size + (current_x - drag.start_pos);
                self.layout.sidebar_width =
                    new_size.clamp(SIDEBAR_MIN, LayoutState::sidebar_max(viewport_width));
            }
            DragTarget::FileColumn => {
                let new_size = drag.start_size + (current_x - drag.start_pos);
                let (sidebar_width, _) = self.layout.fitted(viewport_width);
                self.layout.file_column_width = new_size.clamp(
                    SECONDARY_PANE_MIN,
                    LayoutState::file_column_max(viewport_width, sidebar_width),
                );
            }
        }
        cx.notify();
    }

    pub(crate) fn end_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(drag) = self.layout.drag.take() {
            match drag.target {
                DragTarget::Sidebar => {
                    let width = self.layout.sidebar_width;
                    crate::app::config::update(cx, move |c| {
                        c.layout.sidebar_width = width;
                    });
                }
                DragTarget::FileColumn => {
                    let width = self.layout.file_column_width;
                    crate::app::config::update(cx, move |c| {
                        c.layout.secondary_pane_width = width;
                    });
                }
            }
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitted_keeps_the_closing_sidebar_in_the_file_column_budget() {
        let mut layout = LayoutState {
            sidebar_width: 380.,
            sidebar_hidden: true,
            sidebar_closing: true,
            file_column_width: SECONDARY_PANE_MAX,
            ..LayoutState::default()
        };
        let viewport = 380. + SECONDARY_PANE_MIN + 2. * RESIZE_HANDLE_WIDTH + PREVIEW_MIN;
        assert_eq!(layout.fitted(viewport).1, SECONDARY_PANE_MIN);
        layout.sidebar_closing = false;
        assert!(layout.fitted(viewport).1 > SECONDARY_PANE_MIN);
    }
}
