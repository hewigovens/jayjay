use gpui::{
    AnyElement, App, CursorStyle, DispatchPhase, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, px, rgb,
};

use crate::app::theme::Theme;

pub(super) struct ImageDiffSplit {
    fraction: f32,
    drag_start: Option<(Point<Pixels>, f32)>,
    dragged: bool,
}

impl ImageDiffSplit {
    const PADDING: f32 = 16.;
    const DIVIDER_WIDTH: f32 = 12.;

    pub(super) fn render(
        path: &str,
        before: AnyElement,
        after: AnyElement,
        theme: &Theme,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let state = window.use_keyed_state(
            SharedString::from(format!("image-diff-split-{path}")),
            cx,
            |_, _| Self {
                fraction: 0.5,
                drag_start: None,
                dragged: false,
            },
        );
        let fraction = state.read(cx).fraction;
        let down_state = state.clone();
        div()
            .relative()
            .flex()
            .flex_row()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .p(px(Self::PADDING))
            .bg(rgb(theme.detail_bg))
            .child(
                div()
                    .flex()
                    .flex_basis(px(0.))
                    .flex_grow(fraction)
                    .min_w_0()
                    .min_h_0()
                    .debug_selector(|| "image-before-column".to_owned())
                    .child(before),
            )
            .child(
                div()
                    .id("image-comparison-divider")
                    .flex_none()
                    .w(px(Self::DIVIDER_WIDTH))
                    .h_full()
                    .flex()
                    .justify_center()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .tooltip(crate::ui::primitives::text_tooltip(
                        "Drag to resize images. Double-click to restore equal widths.",
                    ))
                    .debug_selector(|| "image-comparison-divider".to_owned())
                    .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _, cx| {
                        down_state.update(cx, |state, _| {
                            state.drag_start = Some((event.position, state.fraction));
                            state.dragged = false;
                        });
                        cx.stop_propagation();
                    })
                    .child(div().w(px(1.)).h_full().bg(rgb(theme.border))),
            )
            .child(
                div()
                    .flex()
                    .flex_basis(px(0.))
                    .flex_grow(1. - fraction)
                    .min_w_0()
                    .min_h_0()
                    .debug_selector(|| "image-after-column".to_owned())
                    .child(after),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let width =
                            f32::from(bounds.size.width) - 2. * Self::PADDING - Self::DIVIDER_WIDTH;
                        let move_state = state.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                            if phase != DispatchPhase::Bubble || !event.dragging() || width <= 0. {
                                return;
                            }
                            move_state.update(cx, |state, cx| {
                                if let Some((origin, fraction)) = state.drag_start {
                                    let movement = event.position - origin;
                                    state.dragged |= movement.magnitude() > 1.;
                                    state.fraction =
                                        (fraction + f32::from(movement.x) / width).clamp(0.1, 0.9);
                                    cx.notify();
                                    cx.stop_propagation();
                                }
                            });
                        });
                        let up_state = state.clone();
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Bubble && event.button == MouseButton::Left {
                                up_state.update(cx, |state, cx| {
                                    if let Some((origin, _)) = state.drag_start.take()
                                        && event.click_count == 2
                                        && !state.dragged
                                        && (event.position - origin).magnitude() <= 1.
                                    {
                                        state.fraction = 0.5;
                                        cx.notify();
                                    }
                                });
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
            .into_any_element()
    }
}
