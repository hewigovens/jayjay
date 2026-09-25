use gpui::{Context, InteractiveElement, MouseButton, MouseMoveEvent, MouseUpEvent};

/// A divider drag in progress; `pane` names what is being resized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PaneDrag<P> {
    pub(crate) pane: P,
    start_x: f32,
    start_width: f32,
}

impl<P> PaneDrag<P> {
    pub(crate) fn new(pane: P, start_x: f32, start_width: f32) -> Self {
        Self {
            pane,
            start_x,
            start_width,
        }
    }

    pub(crate) fn width_at(&self, x: f32, min: f32, max: f32) -> f32 {
        (self.start_width + x - self.start_x).clamp(min, max)
    }
}

/// The largest width a pane may take from `room`, never below its minimum.
pub(crate) fn pane_max(min: f32, max: f32, room: f32) -> f32 {
    max.min(room.max(min))
}

/// Follows the pointer across the whole window once a resize handle starts a drag.
pub(crate) trait TrackPaneDrag: InteractiveElement + Sized {
    fn track_pane_drag<V: 'static>(
        self,
        on_move: impl Fn(&mut V, f32, f32, &mut Context<V>) + 'static,
        on_end: impl Fn(&mut V, &mut Context<V>) + 'static,
        cx: &mut Context<V>,
    ) -> Self {
        self.on_mouse_move(cx.listener(move |view, ev: &MouseMoveEvent, window, cx| {
            let viewport_width = f32::from(window.viewport_size().width);
            on_move(view, f32::from(ev.position.x), viewport_width, cx);
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |view, _: &MouseUpEvent, _, cx| on_end(view, cx)),
        )
    }
}

impl<E: InteractiveElement> TrackPaneDrag for E {}
