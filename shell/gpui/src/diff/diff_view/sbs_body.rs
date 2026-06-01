use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseUpEvent, ParentElement,
    Pixels, Styled, UniformList, UniformListScrollHandle, div, px, rgb, uniform_list,
};
use jayjay_core::diff::FileDiff;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use super::mouse::{attach_selection_handlers, bounds_capture};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::SbsSide;
use crate::diff::line::ROW_HEIGHT;
use crate::diff::side_by_side::{
    SBS_GUTTER_WIDTH, sbs_new_content, sbs_new_gutter, sbs_old_content, sbs_old_gutter,
};
use crate::diff::wrap::{
    WrappedSbsRow, selection_cols_in_fragment, wrap_cols_from_bounds, wrap_sbs_rows,
};
use crate::repo::window::{PanelBoundsSlot, RepoWindow};
use crate::ui::primitives::no_scrollbar_gutter;
use crate::ui::scrollbar::vertical_uniform_scrollbar;

pub(super) fn side_by_side_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
    old_bounds: PanelBoundsSlot,
    new_bounds: PanelBoundsSlot,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let advance = fonts::mono_advance(cx, px(12.));
    let rows = build_side_by_side_rows(&fd.lines);
    let old_cols = wrap_cols_from_bounds(old_bounds.get(), advance);
    let new_cols = wrap_cols_from_bounds(new_bounds.get(), advance);
    let rows: Arc<Vec<_>> = Arc::new(wrap_sbs_rows(&rows, old_cols, new_cols));
    let count = rows.len();

    let old_gutter = {
        let rows = rows.clone();
        let theme = theme.clone();
        uniform_list(
            "sbs-old-gutter",
            count,
            move |range: std::ops::Range<usize>, _window, _cx| {
                range
                    .map(|ix| sbs_old_gutter(&rows[ix].row, &theme))
                    .collect()
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
                range
                    .map(|ix| sbs_new_gutter(&rows[ix].row, &theme))
                    .collect()
            },
        )
        .track_scroll(&scroll)
    };

    let old_content = sbs_content_list(
        SbsContentArgs {
            id: "sbs-old-content",
            count,
            rows: rows.clone(),
            theme: theme.clone(),
            query: query.clone(),
            scroll: scroll.clone(),
            side: SbsSide::Old,
            bounds: old_bounds.clone(),
            advance,
        },
        cx,
    );
    let new_content = sbs_content_list(
        SbsContentArgs {
            id: "sbs-new-content",
            count,
            rows,
            theme: theme.clone(),
            query,
            scroll: scroll.clone(),
            side: SbsSide::New,
            bounds: new_bounds.clone(),
            advance,
        },
        cx,
    );

    let gutter_panel = |list: UniformList| {
        div()
            .flex_none()
            .w(px(SBS_GUTTER_WIDTH))
            .h_full()
            .border_r_1()
            .border_color(rgb(theme.border))
            .child(no_scrollbar_gutter(list).h_full())
    };
    let content_panel = |list: UniformList,
                         bounds: PanelBoundsSlot,
                         side: SbsSide,
                         show_scrollbar: bool,
                         cx: &mut Context<RepoWindow>| {
        let mut panel = div()
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(bounds_capture(bounds.clone()))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |v, _: &MouseUpEvent, _, cx| {
                    if v.diff
                        .selection
                        .is_some_and(|s| s.side == side && s.dragging)
                    {
                        v.finish_diff_selection(cx);
                    }
                }),
            )
            .child(no_scrollbar_gutter(list).h_full());
        if show_scrollbar {
            panel = panel.child(vertical_uniform_scrollbar(
                scroll.clone(),
                bounds,
                px(count as f32 * ROW_HEIGHT),
                theme.as_ref(),
                cx,
            ));
        }
        panel
    };

    div()
        .flex()
        .flex_row()
        .h_full()
        .min_h_0()
        .child(gutter_panel(old_gutter))
        .child(content_panel(
            old_content,
            old_bounds,
            SbsSide::Old,
            false,
            cx,
        ))
        .child(div().flex_none().w(px(1.)).h_full().bg(rgb(theme.border)))
        .child(gutter_panel(new_gutter))
        .child(content_panel(
            new_content,
            new_bounds,
            SbsSide::New,
            true,
            cx,
        ))
        .into_any_element()
}

struct SbsContentArgs {
    id: &'static str,
    count: usize,
    rows: Arc<Vec<WrappedSbsRow>>,
    theme: Arc<Theme>,
    query: Arc<Option<String>>,
    scroll: UniformListScrollHandle,
    side: SbsSide,
    bounds: PanelBoundsSlot,
    advance: Pixels,
}

fn sbs_content_list(args: SbsContentArgs, cx: &mut Context<RepoWindow>) -> UniformList {
    let SbsContentArgs {
        id,
        count,
        rows,
        theme,
        query,
        scroll,
        side,
        bounds,
        advance,
    } = args;
    uniform_list(
        id,
        count,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let sel = view.diff.selection;
            range
                .map(|ix| {
                    let row = &rows[ix];
                    // Shared wrap records use u32; GPUI selection geometry is usize.
                    let view = if matches!(side, SbsSide::Old) {
                        &row.old
                    } else {
                        &row.new
                    };
                    let line_len = view.line_len as usize;
                    let (col_start, col_end) = (view.col_start as usize, view.col_end as usize);
                    let row_ix = row.row_ix as usize;
                    let selection_cols = sel.and_then(|s| {
                        if s.side == side {
                            s.col_range_for(row_ix, line_len).and_then(|cols| {
                                selection_cols_in_fragment(cols, col_start, col_end)
                            })
                        } else {
                            None
                        }
                    });
                    let cell = if matches!(side, SbsSide::Old) {
                        sbs_old_content(&row.row, &theme, query.as_deref())
                    } else {
                        sbs_new_content(&row.row, &theme, query.as_deref())
                    };
                    // Wrap so the absolute selection overlay has a relative parent.
                    let cell = if let Some(cols) = selection_cols {
                        div()
                            .relative()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .child(cell)
                            .child(crate::diff::line::selection_overlay(cols, advance, &theme))
                    } else {
                        div().flex_1().min_w_0().h_full().child(cell)
                    };
                    attach_selection_handlers(
                        cell,
                        row_ix,
                        side,
                        advance,
                        col_start,
                        bounds.clone(),
                        cx,
                    )
                    .into_any_element()
                })
                .collect()
        }),
    )
    .track_scroll(&scroll)
}
