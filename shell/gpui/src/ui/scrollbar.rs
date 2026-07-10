use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnyElement, Bounds, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Pixels, ScrollHandle, Styled, UniformListScrollHandle, div,
    point, px, rgba,
};

use crate::app::theme::Theme;

pub type ScrollbarBoundsSlot = Rc<Cell<Option<Bounds<Pixels>>>>;

const TRACK_WIDTH: f32 = 10.;
const THUMB_WIDTH: f32 = 4.;
const THUMB_INSET: f32 = 3.;
const MIN_THUMB_HEIGHT: f32 = 32.;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScrollbarGeometry {
    thumb_top: f32,
    thumb_height: f32,
}

#[derive(Debug, Clone, Copy)]
struct ScrollMetrics {
    viewport_height: f32,
    content_height: f32,
    scroll_y: f32,
}

/// Scrollbar for `uniform_list` row content, whose total height is known up front (row count × row height) rather than measured off a tracked handle.
pub fn vertical_uniform_scrollbar<T: 'static>(
    scroll: UniformListScrollHandle,
    bounds: ScrollbarBoundsSlot,
    content_height: Pixels,
    theme: &Theme,
    cx: &mut Context<T>,
) -> AnyElement {
    let Some(metrics) = scroll_metrics(&scroll, &bounds, content_height) else {
        return div().into_any_element();
    };
    let Some(geometry) = scrollbar_geometry(metrics) else {
        return div().into_any_element();
    };
    let set_scroll_y = move |window_y: Pixels| {
        set_scroll_from_window_y(&scroll, &bounds, content_height, window_y);
    };
    scrollbar_widget(geometry, theme, cx, set_scroll_y)
}

/// Scrollbar for free-form `overflow_y_scroll` content tracked by a plain `ScrollHandle`, whose viewport bounds and scrolled extent come straight from the handle.
pub fn vertical_scrollbar<T: 'static>(
    scroll: ScrollHandle,
    theme: &Theme,
    cx: &Context<T>,
) -> AnyElement {
    let Some(metrics) = handle_metrics(&scroll) else {
        return div().into_any_element();
    };
    let Some(geometry) = scrollbar_geometry(metrics) else {
        return div().into_any_element();
    };
    let set_scroll_y = move |window_y: Pixels| set_handle_scroll_from_window_y(&scroll, window_y);
    scrollbar_widget(geometry, theme, cx, set_scroll_y)
}

fn scrollbar_widget<T: 'static>(
    geometry: ScrollbarGeometry,
    theme: &Theme,
    cx: &Context<T>,
    set_scroll_y: impl Fn(Pixels) + Clone + 'static,
) -> AnyElement {
    let track_color = rgba(((theme.border as u64) << 8) as u32 | 0x66);
    let thumb_color = rgba(((theme.fg_faint as u64) << 8) as u32 | 0xd0);
    let down_set = set_scroll_y.clone();
    let move_set = set_scroll_y;

    div()
        .absolute()
        .top(px(0.))
        .right(px(0.))
        .w(px(TRACK_WIDTH))
        .h_full()
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_view, ev: &MouseDownEvent, _window, cx| {
                down_set(ev.position.y);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(move |_view, ev: &MouseMoveEvent, _window, cx| {
            if ev.dragging() {
                move_set(ev.position.y);
                cx.notify();
            }
        }))
        .child(
            div()
                .absolute()
                .top(px(0.))
                .right(px(THUMB_INSET))
                .w(px(THUMB_WIDTH))
                .h_full()
                .rounded_full()
                .bg(track_color),
        )
        .child(
            div()
                .absolute()
                .top(px(geometry.thumb_top))
                .right(px(THUMB_INSET))
                .w(px(THUMB_WIDTH))
                .h(px(geometry.thumb_height))
                .rounded_full()
                .bg(thumb_color),
        )
        .into_any_element()
}

fn scroll_metrics(
    scroll: &UniformListScrollHandle,
    bounds: &ScrollbarBoundsSlot,
    content_height: Pixels,
) -> Option<ScrollMetrics> {
    let viewport_height = f32::from(bounds.get()?.size.height);
    let content_height = f32::from(content_height);
    let scroll_y = -f32::from(scroll.0.borrow().base_handle.offset().y);
    Some(ScrollMetrics {
        viewport_height,
        content_height,
        scroll_y,
    })
}

fn handle_metrics(scroll: &ScrollHandle) -> Option<ScrollMetrics> {
    let viewport_height = f32::from(scroll.bounds().size.height);
    if viewport_height <= 0. {
        return None;
    }
    let content_height = viewport_height + f32::from(scroll.max_offset().y);
    let scroll_y = -f32::from(scroll.offset().y);
    Some(ScrollMetrics {
        viewport_height,
        content_height,
        scroll_y,
    })
}

fn scrollbar_geometry(metrics: ScrollMetrics) -> Option<ScrollbarGeometry> {
    if metrics.content_height <= metrics.viewport_height || metrics.viewport_height <= 0. {
        return None;
    }

    let max_scroll = metrics.content_height - metrics.viewport_height;
    let scroll_y = metrics.scroll_y.clamp(0., max_scroll);
    let thumb_height = (metrics.viewport_height / metrics.content_height * metrics.viewport_height)
        .clamp(MIN_THUMB_HEIGHT, metrics.viewport_height);
    let max_thumb_top = (metrics.viewport_height - thumb_height).max(0.);
    let thumb_top = if max_scroll <= 0. {
        0.
    } else {
        scroll_y / max_scroll * max_thumb_top
    };

    Some(ScrollbarGeometry {
        thumb_top,
        thumb_height,
    })
}

fn set_scroll_from_window_y(
    scroll: &UniformListScrollHandle,
    bounds: &ScrollbarBoundsSlot,
    content_height: Pixels,
    window_y: Pixels,
) {
    let Some(container_top) = bounds.get().map(|b| b.origin.y) else {
        return;
    };
    let Some(metrics) = scroll_metrics(scroll, bounds, content_height) else {
        return;
    };
    let Some(geometry) = scrollbar_geometry(metrics) else {
        return;
    };

    let base = scroll.0.borrow().base_handle.clone();
    let offset = base.offset();
    let scroll_y = scroll_y_for_window_y(metrics, geometry, container_top, window_y);
    base.set_offset(point(offset.x, scroll_y));
}

fn set_handle_scroll_from_window_y(scroll: &ScrollHandle, window_y: Pixels) {
    let Some(metrics) = handle_metrics(scroll) else {
        return;
    };
    let Some(geometry) = scrollbar_geometry(metrics) else {
        return;
    };

    let offset = scroll.offset();
    let scroll_y = scroll_y_for_window_y(metrics, geometry, scroll.bounds().origin.y, window_y);
    scroll.set_offset(point(offset.x, scroll_y));
}

/// Maps a drag/click position on the track to the scroll offset that puts the thumb there.
fn scroll_y_for_window_y(
    metrics: ScrollMetrics,
    geometry: ScrollbarGeometry,
    container_top: Pixels,
    window_y: Pixels,
) -> Pixels {
    let local_y =
        (f32::from(window_y) - f32::from(container_top)).clamp(0., metrics.viewport_height);
    let max_thumb_top = (metrics.viewport_height - geometry.thumb_height).max(0.);
    let thumb_top = (local_y - geometry.thumb_height / 2.).clamp(0., max_thumb_top);
    let max_scroll = metrics.content_height - metrics.viewport_height;
    let scroll_y = if max_thumb_top <= 0. {
        0.
    } else {
        thumb_top / max_thumb_top * max_scroll
    };
    px(-scroll_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_scrollbar_when_content_fits() {
        assert_eq!(
            scrollbar_geometry(ScrollMetrics {
                viewport_height: 200.,
                content_height: 200.,
                scroll_y: 0.,
            }),
            None
        );
    }

    #[test]
    fn scales_thumb_and_maps_offset() {
        let geometry = scrollbar_geometry(ScrollMetrics {
            viewport_height: 200.,
            content_height: 1000.,
            scroll_y: 400.,
        })
        .expect("scrollbar geometry");

        assert_eq!(geometry.thumb_height, 40.);
        assert_eq!(geometry.thumb_top, 80.);
    }
}
