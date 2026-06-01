use gpui::Context;

use super::{
    ColumnDrag, DESCRIPTION_MAX, DESCRIPTION_MIN, DragTarget, FILE_COLUMN_MAX, FILE_COLUMN_MIN,
    RepoWindow, SIDEBAR_MAX, SIDEBAR_MIN,
};

impl RepoWindow {
    pub(crate) fn start_drag(
        &mut self,
        target: DragTarget,
        start_pos: f32,
        cx: &mut Context<Self>,
    ) {
        let start_size = match target {
            DragTarget::Sidebar => self.layout.sidebar_width,
            DragTarget::FileColumn => self.layout.file_column_width,
            DragTarget::Description => self.layout.description_height,
        };
        self.layout.drag = Some(ColumnDrag {
            target,
            start_pos,
            start_size,
        });
        cx.notify();
    }

    pub(crate) fn drag_to(&mut self, current_x: f32, current_y: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.layout.drag else {
            return;
        };
        match drag.target {
            DragTarget::Sidebar => {
                let new_size = drag.start_size + (current_x - drag.start_pos);
                self.layout.sidebar_width = new_size.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
            }
            DragTarget::FileColumn => {
                let new_size = drag.start_size + (current_x - drag.start_pos);
                self.layout.file_column_width = new_size.clamp(FILE_COLUMN_MIN, FILE_COLUMN_MAX);
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
                DragTarget::Description => {
                    let height = self.layout.description_height;
                    crate::app::config::update(cx, move |c| {
                        c.layout.description_height = height;
                    });
                }
                DragTarget::FileColumn => {} // session-only
            }
        }
        cx.notify();
    }
}
