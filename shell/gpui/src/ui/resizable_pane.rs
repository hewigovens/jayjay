use super::pane_drag::{PaneDrag, pane_max};
use super::resize_handle::RESIZE_HANDLE_WIDTH;

/// A pane sized by one divider that leaves at least `reserved` for the rest of the window.
pub(crate) struct ResizablePane {
    width: f32,
    min: f32,
    max: f32,
    reserved: f32,
    drag: Option<PaneDrag<()>>,
}

impl ResizablePane {
    pub(crate) fn new(width: f32, min: f32, max: f32, reserved: f32) -> Self {
        Self {
            width: width.clamp(min, max),
            min,
            max,
            reserved,
            drag: None,
        }
    }

    pub(crate) fn fitted(&self, viewport_width: f32) -> f32 {
        self.width.min(self.max_width(viewport_width))
    }

    fn max_width(&self, viewport_width: f32) -> f32 {
        let room = viewport_width - RESIZE_HANDLE_WIDTH - self.reserved;
        pane_max(self.min, self.max, room)
    }

    pub(crate) fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub(crate) fn start_drag(&mut self, x: f32, viewport_width: f32) {
        self.drag = Some(PaneDrag::new((), x, self.fitted(viewport_width)));
    }

    pub(crate) fn drag_to(&mut self, x: f32, viewport_width: f32) -> bool {
        let Some(drag) = self.drag else {
            return false;
        };
        self.width = drag.width_at(x, self.min, self.max_width(viewport_width));
        true
    }

    pub(crate) fn end_drag(&mut self) -> Option<f32> {
        self.drag.take().map(|_| self.width)
    }
}
