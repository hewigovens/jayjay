use gpui::Context;

use super::{
    ColumnDrag, DESCRIPTION_MAX, DESCRIPTION_MIN, DragTarget, FILE_COLUMN_MAX, FILE_COLUMN_MIN,
    LayoutState, PREVIEW_MIN, RESIZE_HANDLE_WIDTH, RepoWindow, SIDEBAR_MAX, SIDEBAR_MIN,
};

impl LayoutState {
    fn sidebar_max(viewport_width: f32) -> f32 {
        let room = viewport_width - FILE_COLUMN_MIN - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        SIDEBAR_MAX.min(room.max(SIDEBAR_MIN))
    }

    fn file_column_max(viewport_width: f32, sidebar_width: f32) -> f32 {
        let room = viewport_width - sidebar_width - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        FILE_COLUMN_MAX.min(room.max(FILE_COLUMN_MIN))
    }

    /// The widths actually shown: persisted maxima can exceed a smaller window.
    pub(crate) fn fitted(&self, viewport_width: f32) -> (f32, f32) {
        let sidebar = self.sidebar_width.min(Self::sidebar_max(viewport_width));
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
            DragTarget::Description => self.layout.description_height,
        };
        self.layout.drag = Some(ColumnDrag {
            target,
            start_pos,
            start_size,
        });
        cx.notify();
    }

    pub(crate) fn drag_to(
        &mut self,
        current_x: f32,
        current_y: f32,
        viewport_width: f32,
        cx: &mut Context<Self>,
    ) {
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
                    FILE_COLUMN_MIN,
                    LayoutState::file_column_max(viewport_width, sidebar_width),
                );
            }
            DragTarget::Description => {
                let new_size = drag.start_size + (current_y - drag.start_pos);
                self.layout.description_height = new_size.clamp(DESCRIPTION_MIN, DESCRIPTION_MAX);
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
                        c.layout.file_column_width = width;
                    });
                }
                DragTarget::Description => {
                    let height = self.layout.description_height;
                    crate::app::config::update(cx, move |c| {
                        c.layout.description_height = height;
                    });
                }
            }
        }
        cx.notify();
    }
}
