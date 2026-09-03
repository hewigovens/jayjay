use gpui::{
    AnyElement, Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;

pub(crate) const RESIZE_HANDLE_WIDTH: f32 = 5.;

/// Vertical divider that starts a horizontal drag; `on_down` gets the pointer x and the viewport width.
pub(crate) fn resize_handle<V: 'static>(
    debug_selector: &'static str,
    t: &Theme,
    on_down: impl Fn(&mut V, f32, f32, &mut Context<V>) + 'static,
    cx: &mut Context<V>,
) -> AnyElement {
    div()
        .flex_none()
        .w(px(RESIZE_HANDLE_WIDTH))
        .h_full()
        .cursor(CursorStyle::ResizeLeftRight)
        .debug_selector(move || debug_selector.to_owned())
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, window, cx| {
                let viewport_width = f32::from(window.viewport_size().width);
                on_down(view, f32::from(ev.position.x), viewport_width, cx);
            }),
        )
        .child(div().w(px(1.)).h_full().ml(px(2.)).bg(rgb(t.border)))
        .into_any_element()
}
