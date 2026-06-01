use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseUpEvent, ParentElement,
    Styled, UniformListScrollHandle, div, px, rgb, uniform_list,
};
use jayjay_core::diff::FileDiff;

use super::mouse::{attach_selection_handlers, bounds_capture};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::SbsSide;
use crate::diff::line::{GUTTER_WIDTH, ROW_HEIGHT, content_row, gutter_row};
use crate::diff::wrap::{selection_cols_in_fragment, wrap_cols_from_bounds, wrap_diff_lines};
use crate::repo::window::{PanelBoundsSlot, RepoWindow};
use crate::ui::primitives::no_scrollbar_gutter;
use crate::ui::scrollbar::vertical_uniform_scrollbar;

pub(super) fn unified_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
    bounds_slot: PanelBoundsSlot,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let advance = fonts::mono_advance(cx, px(12.));
    let wrap_cols = wrap_cols_from_bounds(bounds_slot.get(), advance);
    let lines: Arc<Vec<_>> = Arc::new(wrap_diff_lines(&fd.lines, wrap_cols));
    let count = lines.len();

    let gutter_lines = lines.clone();
    let gutter_theme = theme.clone();
    let gutter = uniform_list(
        "diff-gutter",
        count,
        move |range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| gutter_row(&gutter_lines[ix].line, &gutter_theme))
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
            let sel = view.diff.selection;
            range
                .map(|ix| {
                    let line = &content_lines[ix];
                    // Shared wrap records use u32; GPUI selection geometry is usize.
                    let line_ix = line.line_ix as usize;
                    let line_len = line.line_len as usize;
                    let col_start = line.col_start as usize;
                    let col_end = line.col_end as usize;
                    let selection_cols = sel.and_then(|s| {
                        if s.side == SbsSide::Unified {
                            s.col_range_for(line_ix, line_len).and_then(|cols| {
                                selection_cols_in_fragment(cols, col_start, col_end)
                            })
                        } else {
                            None
                        }
                    });
                    let row = content_row(
                        &line.line,
                        &content_theme,
                        content_query.as_deref(),
                        selection_cols,
                        advance,
                    );
                    attach_selection_handlers(
                        row,
                        line_ix,
                        SbsSide::Unified,
                        advance,
                        col_start,
                        content_bounds.clone(),
                        cx,
                    )
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
                .child(no_scrollbar_gutter(gutter).h_full()),
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
                .child(no_scrollbar_gutter(content).h_full())
                .child(vertical_uniform_scrollbar(
                    scroll,
                    bounds_slot,
                    px(count as f32 * ROW_HEIGHT),
                    theme.as_ref(),
                    cx,
                )),
        )
        .into_any_element()
}
