use gpui::Context;

use super::{
    DragTarget, LayoutState, OPERATION_LOG_MAX, OPERATION_LOG_MIN, PREVIEW_MIN, RepoWindow,
    SECONDARY_PANE_MAX, SECONDARY_PANE_MIN, SIDEBAR_MAX, SIDEBAR_MIN,
};
use crate::ui::pane_drag::{PaneDrag, pane_max};
use crate::ui::resize_handle::RESIZE_HANDLE_WIDTH;

impl LayoutState {
    fn sidebar_shown(&self) -> bool {
        !self.sidebar_hidden || self.sidebar_closing
    }

    fn operation_log_max(&self, viewport_width: f32) -> f32 {
        let sidebar = if self.sidebar_shown() {
            SIDEBAR_MIN + RESIZE_HANDLE_WIDTH
        } else {
            0.
        };
        let room =
            viewport_width - sidebar - SECONDARY_PANE_MIN - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        room.min(OPERATION_LOG_MAX)
    }

    /// Open or sliding closed; narrows, then hides, so the other panes keep their minimums.
    pub(crate) fn operation_log_pane_width(&self, viewport_width: f32) -> Option<f32> {
        let max = self.operation_log_max(viewport_width);
        ((self.operation_log_shown || self.operation_log_closing) && max >= OPERATION_LOG_MIN)
            .then(|| self.operation_log_width.min(max))
    }

    pub(crate) fn operation_log_visible(&self) -> bool {
        self.operation_log_shown && self.operation_log_pane_width(self.viewport_width).is_some()
    }

    fn content_width(&self, viewport_width: f32) -> f32 {
        self.operation_log_pane_width(viewport_width)
            .map_or(viewport_width, |pane| {
                viewport_width - pane - RESIZE_HANDLE_WIDTH
            })
    }

    /// The width between the sidebar and the operation log.
    pub(crate) fn centre_width(&self, viewport_width: f32) -> f32 {
        let (sidebar, _) = self.fitted(viewport_width);
        let sidebar = if sidebar > 0. {
            sidebar + RESIZE_HANDLE_WIDTH
        } else {
            0.
        };
        (self.content_width(viewport_width) - sidebar).max(0.)
    }

    fn sidebar_max(viewport_width: f32) -> f32 {
        let room = viewport_width - SECONDARY_PANE_MIN - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        pane_max(SIDEBAR_MIN, SIDEBAR_MAX, room)
    }

    pub(crate) fn sidebar_pane_width(&self, viewport_width: f32) -> f32 {
        self.sidebar_width
            .min(Self::sidebar_max(self.content_width(viewport_width)))
    }

    fn file_column_max(viewport_width: f32, sidebar_width: f32) -> f32 {
        let room = viewport_width - sidebar_width - 2. * RESIZE_HANDLE_WIDTH - PREVIEW_MIN;
        pane_max(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX, room)
    }

    pub(crate) fn fitted(&self, viewport_width: f32) -> (f32, f32) {
        let sidebar = if self.sidebar_shown() {
            self.sidebar_pane_width(viewport_width)
        } else {
            0.
        };
        let file_column = self.file_column_width.min(Self::file_column_max(
            self.content_width(viewport_width),
            sidebar,
        ));
        (sidebar, file_column)
    }
}

impl RepoWindow {
    pub(crate) fn start_drag(
        &mut self,
        target: DragTarget,
        start_x: f32,
        viewport_width: f32,
        cx: &mut Context<Self>,
    ) {
        let (sidebar_width, file_column_width) = self.layout.fitted(viewport_width);
        let start_width = match target {
            DragTarget::Sidebar => sidebar_width,
            DragTarget::FileColumn => file_column_width,
            DragTarget::OperationLog => self
                .layout
                .operation_log_pane_width(viewport_width)
                .unwrap_or(self.layout.operation_log_width),
        };
        self.layout.drag = Some(PaneDrag::new(target, start_x, start_width));
        cx.notify();
    }

    pub(crate) fn drag_to(&mut self, current_x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.layout.drag else {
            return;
        };
        match drag.pane {
            DragTarget::Sidebar => {
                self.layout.sidebar_width = drag.width_at(
                    current_x,
                    SIDEBAR_MIN,
                    LayoutState::sidebar_max(self.layout.content_width(viewport_width)),
                );
            }
            DragTarget::FileColumn => {
                let (sidebar_width, _) = self.layout.fitted(viewport_width);
                self.layout.file_column_width = drag.width_at(
                    current_x,
                    SECONDARY_PANE_MIN,
                    LayoutState::file_column_max(
                        self.layout.content_width(viewport_width),
                        sidebar_width,
                    ),
                );
            }
            DragTarget::OperationLog => {
                self.layout.operation_log_width = drag.trailing_width_at(
                    current_x,
                    OPERATION_LOG_MIN,
                    self.layout.operation_log_max(viewport_width),
                );
            }
        }
        cx.notify();
    }

    pub(crate) fn end_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(drag) = self.layout.drag.take() {
            match drag.pane {
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
                DragTarget::OperationLog => {
                    let width = self.layout.operation_log_width;
                    crate::app::config::update(cx, move |c| {
                        c.layout.operation_log_width = width;
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

    #[test]
    fn the_operation_log_leaves_every_other_pane_its_minimum() {
        let layout = LayoutState {
            sidebar_width: SIDEBAR_MAX,
            file_column_width: SECONDARY_PANE_MAX,
            operation_log_shown: true,
            operation_log_width: OPERATION_LOG_MAX,
            ..LayoutState::default()
        };
        let viewport = 1200.;
        let (sidebar, file_column) = layout.fitted(viewport);
        let pane = layout
            .operation_log_pane_width(viewport)
            .expect("pane fits");
        let preview = viewport - sidebar - file_column - pane - 3. * RESIZE_HANDLE_WIDTH;
        assert!(sidebar >= SIDEBAR_MIN && file_column >= SECONDARY_PANE_MIN);
        assert!(preview >= PREVIEW_MIN, "preview squeezed to {preview}");
        assert!(pane < OPERATION_LOG_MAX);
        assert_eq!(
            layout.operation_log_pane_width(4000.),
            Some(OPERATION_LOG_MAX)
        );

        let narrow = 640.;
        assert_eq!(layout.operation_log_pane_width(narrow), None);
        assert!(layout.fitted(narrow).1 >= SECONDARY_PANE_MIN);
        assert_eq!(layout.operation_log_width, OPERATION_LOG_MAX);
    }
}
