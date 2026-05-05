use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Styled, UniformListScrollHandle, canvas,
    div, px, rgb, uniform_list,
};
use jayjay_core::diff::FileDiff;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use crate::app::theme::Theme;
use crate::diff::SbsSide;
use crate::diff::line::{GUTTER_WIDTH, MONO_GLYPH_WIDTH, content_row, gutter_row};
use crate::diff::side_by_side::{
    SBS_GUTTER_WIDTH, sbs_new_content, sbs_new_gutter, sbs_old_content, sbs_old_gutter,
};
use crate::log::{LogView, PanelBoundsSlot};

pub(super) fn unified_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let lines: Arc<Vec<jayjay_core::diff::DiffLine>> = Arc::new(fd.lines.clone());
    let count = lines.len();
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let bounds_slot = cx.entity().read(cx).diff_unified_bounds.clone();

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
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener({
                            let bounds = content_bounds.clone();
                            move |v, ev: &MouseDownEvent, _, cx| {
                                let col = pixel_to_col(&bounds, ev.position.x);
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
                            let col = pixel_to_col(&bounds, ev.position.x);
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

pub(super) fn side_by_side_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let rows: Arc<Vec<_>> = Arc::new(build_side_by_side_rows(&fd.lines));
    let count = rows.len();
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let old_bounds = cx.entity().read(cx).diff_sbs_old_bounds.clone();
    let new_bounds = cx.entity().read(cx).diff_sbs_new_bounds.clone();

    let old_gutter = {
        let rows = rows.clone();
        let theme = theme.clone();
        uniform_list(
            "sbs-old-gutter",
            count,
            move |range: std::ops::Range<usize>, _window, _cx| {
                range.map(|ix| sbs_old_gutter(&rows[ix], &theme)).collect()
            },
        )
        .track_scroll(&scroll)
    };
    let new_gutter = {
        let rows = rows.clone();
        let theme = theme.clone();
        uniform_list(
            "sbs-new-gutter",
            count,
            move |range: std::ops::Range<usize>, _window, _cx| {
                range.map(|ix| sbs_new_gutter(&rows[ix], &theme)).collect()
            },
        )
        .track_scroll(&scroll)
    };

    let old_content = sbs_content_list(
        "sbs-old-content",
        count,
        rows.clone(),
        theme.clone(),
        query.clone(),
        scroll.clone(),
        SbsSide::Old,
        old_bounds.clone(),
        cx,
    );
    let new_content = sbs_content_list(
        "sbs-new-content",
        count,
        rows,
        theme.clone(),
        query,
        scroll,
        SbsSide::New,
        new_bounds.clone(),
        cx,
    );

    let gutter_panel = |list: gpui::UniformList| {
        div()
            .flex_none()
            .w(px(SBS_GUTTER_WIDTH))
            .h_full()
            .border_r_1()
            .border_color(rgb(theme.border))
            .child(crate::ui::primitives::no_scrollbar_gutter(list).h_full())
    };
    let content_panel = |list: gpui::UniformList,
                         bounds: PanelBoundsSlot,
                         side: SbsSide,
                         cx: &mut Context<LogView>| {
        div()
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(bounds_capture(bounds))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |v, _: &MouseUpEvent, _, cx| {
                    if v.diff_selection
                        .is_some_and(|s| s.side == side && s.dragging)
                    {
                        v.finish_diff_selection(cx);
                    }
                }),
            )
            .child(crate::ui::primitives::no_scrollbar_gutter(list).h_full())
    };

    div()
        .flex()
        .flex_row()
        .h_full()
        .min_h_0()
        .child(gutter_panel(old_gutter))
        .child(content_panel(old_content, old_bounds, SbsSide::Old, cx))
        .child(div().flex_none().w(px(1.)).h_full().bg(rgb(theme.border)))
        .child(gutter_panel(new_gutter))
        .child(content_panel(new_content, new_bounds, SbsSide::New, cx))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn sbs_content_list(
    id: &'static str,
    count: usize,
    rows: Arc<Vec<jayjay_core::diff::side_by_side::SideBySideRow>>,
    theme: Arc<Theme>,
    query: Arc<Option<String>>,
    scroll: UniformListScrollHandle,
    side: SbsSide,
    bounds: PanelBoundsSlot,
    cx: &mut Context<LogView>,
) -> gpui::UniformList {
    uniform_list(
        id,
        count,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let sel = view.diff_selection;
            range
                .map(|ix| {
                    let row = &rows[ix];
                    let spans = if matches!(side, SbsSide::Old) {
                        &row.old_spans
                    } else {
                        &row.new_spans
                    };
                    let line_len = spans.iter().map(|s| s.text.chars().count()).sum();
                    let selection_cols = sel.and_then(|s| {
                        if s.side == side {
                            s.col_range_for(ix, line_len)
                        } else {
                            None
                        }
                    });
                    let cell = if matches!(side, SbsSide::Old) {
                        sbs_old_content(row, &theme, query.as_deref())
                    } else {
                        sbs_new_content(row, &theme, query.as_deref())
                    };
                    // Apply selection overlay manually via a relative wrapper
                    // because sbs_*_content returns a Div without space for
                    // the absolute overlay child.
                    let cell = if let Some(cols) = selection_cols {
                        div()
                            .relative()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .child(crate::diff::line::selection_overlay(cols, &theme))
                            .child(cell)
                    } else {
                        div().flex_1().min_w_0().h_full().child(cell)
                    };
                    cell.on_mouse_down(
                        MouseButton::Left,
                        cx.listener({
                            let bounds = bounds.clone();
                            move |v, ev: &MouseDownEvent, _, cx| {
                                let col = pixel_to_col(&bounds, ev.position.x);
                                if ev.click_count >= 2 {
                                    v.select_word(ix, col, side, cx);
                                } else {
                                    v.start_diff_selection(ix, col, side, cx);
                                }
                            }
                        }),
                    )
                    .on_mouse_move(cx.listener({
                        let bounds = bounds.clone();
                        move |v, ev: &MouseMoveEvent, _, cx| {
                            let col = pixel_to_col(&bounds, ev.position.x);
                            v.extend_diff_selection(ix, col, side, cx);
                        }
                    }))
                    .into_any_element()
                })
                .collect()
        }),
    )
    .track_scroll(&scroll)
}

/// `gpui::canvas` overlay that captures the panel's bounds during prepaint.
/// Sized full and absolute so it overlays without consuming layout.
fn bounds_capture(slot: PanelBoundsSlot) -> impl IntoElement {
    canvas(
        move |bounds, _window, _cx| {
            slot.set(Some(bounds));
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}

fn pixel_to_col(slot: &PanelBoundsSlot, x: Pixels) -> usize {
    let Some(bounds) = slot.get() else {
        return 0;
    };
    let local = (f32::from(x) - f32::from(bounds.origin.x)).max(0.);
    (local / MONO_GLYPH_WIDTH).floor() as usize
}
