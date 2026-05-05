use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Styled, UniformListScrollHandle, div, px, rgb,
    uniform_list,
};
use jayjay_core::diff::FileDiff;

use super::mouse::{bounds_capture, pixel_to_col};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::SbsSide;
use crate::diff::line::{GUTTER_WIDTH, content_row, gutter_row};
use crate::log::{LogView, PanelBoundsSlot};

pub(super) fn unified_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
    bounds_slot: PanelBoundsSlot,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let lines: Arc<Vec<jayjay_core::diff::DiffLine>> = Arc::new(fd.lines.clone());
    let count = lines.len();
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let advance = fonts::mono_advance(cx, px(12.));

    let gutter_lines = lines.clone();
    let gutter_theme = theme.clone();
    let gutter = uniform_list(
        "diff-gutter",
        count,
        move |range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| gutter_row(&gutter_lines[ix], &gutter_theme))
                .collect()
        },
    )
    .track_scroll(&scroll);

    let content_lines = lines;
    let content_theme = theme.clone();
    let content_query = query;
    let content_bounds = bounds_slot.clone();
    let content = uniform_list(
        "diff-content",
        count,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let sel = view.diff_selection;
            range
                .map(|ix| {
                    let line = &content_lines[ix];
                    let line_len = line.spans.iter().map(|s| s.text.chars().count()).sum();
                    let selection_cols = sel.and_then(|s| {
                        if s.side == SbsSide::Unified {
                            s.col_range_for(ix, line_len)
                        } else {
                            None
                        }
                    });
                    content_row(
                        line,
                        &content_theme,
                        content_query.as_deref(),
                        selection_cols,
                        advance,
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener({
                            let bounds = content_bounds.clone();
                            move |v, ev: &MouseDownEvent, _, cx| {
                                let col = pixel_to_col(&bounds, ev.position.x, advance);
                                if ev.click_count >= 2 {
                                    v.select_word(ix, col, SbsSide::Unified, cx);
                                } else {
                                    v.start_diff_selection(ix, col, SbsSide::Unified, cx);
                                }
                            }
                        }),
                    )
                    .on_mouse_move(cx.listener({
                        let bounds = content_bounds.clone();
                        move |v, ev: &MouseMoveEvent, _, cx| {
                            let col = pixel_to_col(&bounds, ev.position.x, advance);
                            v.extend_diff_selection(ix, col, SbsSide::Unified, cx);
                        }
                    }))
                    .into_any_element()
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);

    div()
        .flex()
        .flex_row()
        .h_full()
        .min_h_0()
        .child(
            div()
                .flex_none()
                .w(px(GUTTER_WIDTH))
                .h_full()
                .border_r_1()
                .border_color(rgb(theme.border))
                .child(crate::ui::primitives::no_scrollbar_gutter(gutter).h_full()),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w_0()
                .h_full()
                .child(bounds_capture(bounds_slot.clone()))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|v, _: &MouseUpEvent, _, cx| {
                        v.finish_diff_selection(cx);
                    }),
                )
                .child(crate::ui::primitives::no_scrollbar_gutter(content).h_full()),
        )
        .into_any_element()
}
