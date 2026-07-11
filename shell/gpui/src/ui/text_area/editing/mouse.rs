use gpui::{Context, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ScrollWheelEvent, Window, px};

use super::super::TextArea;

impl TextArea {
    pub(in crate::ui::text_area) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        self.is_selecting = true;
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    pub(in crate::ui::text_area) fn on_mouse_up(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.is_selecting = false;
    }

    pub(in crate::ui::text_area) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    pub(in crate::ui::text_area) fn on_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.multiline {
            return;
        }
        let (Some(bounds), Some(layout)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return;
        };
        let content_height = px(f32::from(layout.line_height) * layout.lines.len() as f32);
        let max_scroll = content_height - bounds.size.height;
        if max_scroll <= px(0.) {
            return;
        }
        let delta_y = event.delta.pixel_delta(window.line_height()).y;
        let next = (self.scroll_y - delta_y).max(px(0.)).min(max_scroll);
        if next == self.scroll_y {
            return;
        }
        self.scroll_y = next;
        cx.stop_propagation();
        cx.notify();
    }
}
