use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Pixels, Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::{FileDiff, WrappedDiffLine};
use jayjay_review::ReviewNoteStatus;

use super::context_controls::context_controls;
use super::gutter_mouse::attach_gutter_selection_handlers;
use super::mouse::{SelectionCell, attach_selection_handlers};
use super::moved_label::moved_label;
use super::review_stripe::review_stripe_cell;
use super::row_anchor::{VisibleRows, scroll_to_row};
use super::row_list::RowPlacement;
use super::rows::{DiffRenderRow, DiffRenderRows};
use super::state::ReviewDisplayState;
use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::diff::line::{
    CODE_INSET, content_row, content_row_tint, gutter_column, interactive_gutter_row,
    line_bg_color, note_content_row, note_dot_cell, note_gutter_row, review_stripe_spacer,
};
use crate::diff::wrap::{selection_cols_in_fragment, wrap_cols_from_bounds};
use crate::diff::{DiffSelection, GutterLineSelection, SbsSide};
use crate::repo::window::{DiffWrapCacheSlot, PanelBoundsSlot, RepoWindow};
use crate::ui::scrollbar::vertical_uniform_scrollbar;

pub(super) struct UnifiedBodyState<'a> {
    pub(super) file_diff: &'a Arc<FileDiff>,
    pub(super) theme: Theme,
    pub(super) query: Option<String>,
    pub(super) placement: RowPlacement,
    pub(super) bounds: PanelBoundsSlot,
    pub(super) wrap_cache: &'a DiffWrapCacheSlot,
    pub(super) notes: &'a [ReviewNoteStatus],
    pub(super) shows_review: bool,
    pub(super) review: Option<Arc<ReviewDisplayState>>,
}

pub(super) fn unified_body(
    state: UnifiedBodyState<'_>,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let UnifiedBodyState {
        file_diff,
        theme,
        query,
        placement,
        bounds,
        wrap_cache,
        notes,
        shows_review,
        review,
    } = state;
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let advance = fonts::mono_advance(cx, px(theme.code_font_size()));
    let wrap_cols = wrap_cols_from_bounds(bounds.get(), advance);
    let label_cols = wrap_cache.borrow_mut().moved_label_cols(file_diff, cx);
    let lines = wrap_cache
        .borrow_mut()
        .unified(file_diff, wrap_cols, label_cols);
    // Both lists size off this shared, interleaved row list — never off lines.len() — so a NoteText row shifts gutter and content lists in lockstep.
    let rendered = wrap_cache
        .borrow_mut()
        .rows(file_diff, wrap_cols, notes, label_cols);
    let count = rendered.rows.len();
    if let Some(scroll) = placement.scroll()
        && let Some(row) = wrap_cache.borrow_mut().show(VisibleRows::Unified {
            diff: file_diff.clone(),
            lines: lines.clone(),
            rendered: rendered.clone(),
        })
    {
        scroll_to_row(scroll, row, theme.code_line_height());
    }
    let row_height = theme.code_line_height();
    let shows_selection = placement.shows_selection();

    let gutter_lines = lines.clone();
    let gutter_rendered = rendered.clone();
    let gutter_theme = theme.clone();
    let gutter_path = file_diff.path.clone();
    let gutter_review = review.clone();
    let gutter = placement.rows(
        "diff-gutter",
        count,
        row_height,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let selection = view.diff.gutter_selection.clone();
            range
                .map(|ix| {
                    gutter_row_at(
                        GutterRowState {
                            ix,
                            rendered: &gutter_rendered,
                            lines: &gutter_lines,
                            path: &gutter_path,
                            selection: selection.as_ref(),
                            theme: &gutter_theme,
                            shows_review,
                            review: gutter_review.as_ref(),
                        },
                        cx,
                    )
                })
                .collect()
        }),
    );

    let content_lines = lines;
    let content_rendered = rendered;
    let content_theme = theme.clone();
    let content_query = query;
    let content_bounds = bounds.clone();
    let content_path = file_diff.path.clone();
    let content = placement.rows(
        "diff-content",
        count,
        row_height,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let sel = view.diff.selection.filter(|_| shows_selection);
            let gutter_sel = view.diff.gutter_selection.clone();
            range
                .map(|ix| {
                    content_row_at(
                        ContentRowState {
                            ix,
                            rendered: &content_rendered,
                            lines: &content_lines,
                            path: &content_path,
                            selection: sel,
                            gutter_selection: gutter_sel.as_ref(),
                            theme: &content_theme,
                            query: content_query.as_deref(),
                            advance,
                            bounds: &content_bounds,
                            extends_selection: shows_selection,
                        },
                        cx,
                    )
                })
                .collect()
        }),
    );

    let scrollbar = placement.scroll().map(|scroll| {
        vertical_uniform_scrollbar(
            scroll.clone(),
            bounds.clone(),
            px(count as f32 * row_height),
            theme.as_ref(),
            cx,
        )
    });
    div()
        .flex()
        .flex_row()
        .h_full()
        .min_h_0()
        .child(
            gutter_column(theme.as_ref(), shows_review)
                .debug_selector(|| "diff-gutter".to_owned())
                .child(gutter),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w_0()
                .h_full()
                .bg(rgb(theme.diff_context_bg))
                .child(placement.bounds_capture(bounds))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|v, _: &MouseUpEvent, _, cx| {
                        v.finish_diff_selection(cx);
                    }),
                )
                .child(content)
                .children(scrollbar),
        )
        .into_any_element()
}

struct GutterRowState<'a> {
    ix: usize,
    rendered: &'a DiffRenderRows,
    lines: &'a [WrappedDiffLine],
    path: &'a str,
    selection: Option<&'a GutterLineSelection>,
    theme: &'a Theme,
    shows_review: bool,
    review: Option<&'a Arc<ReviewDisplayState>>,
}

fn gutter_row_at(state: GutterRowState<'_>, cx: &mut Context<RepoWindow>) -> AnyElement {
    let GutterRowState {
        ix,
        rendered,
        lines,
        path,
        selection,
        theme,
        shows_review,
        review,
    } = state;
    let DiffRenderRow::Line(w_ix) = &rendered.rows[ix] else {
        return note_gutter_row(theme, shows_review).into_any_element();
    };
    let w_ix = *w_ix;
    let line = &lines[w_ix];
    if line.line.context_region.is_some() {
        return interactive_gutter_row(
            &line.line,
            theme,
            false,
            shows_review.then(|| review_stripe_spacer().into_any_element()),
            note_dot_cell(None, theme, theme.diff_separator_bg).into_any_element(),
        )
        .into_any_element();
    }
    let line_ix = line.line_ix as usize;
    let is_selected = selection.is_some_and(|sel| sel.covers(path, line_ix));
    let dot = rendered.dots.get(&w_ix).copied();
    let line_bg = line_bg_color(line.line.style, line.line.conflict_kind, theme);
    let review_cell = shows_review.then(|| {
        review_stripe_cell(
            ("review-hunk-unified", ix),
            review,
            review.and_then(|review| review.unified_group(line_ix)),
            theme,
            cx,
        )
    });
    let mut dot_cell = note_dot_cell(dot, theme, line_bg);
    if dot.is_some() {
        let dot_path = path.to_owned();
        let hover_bg = with_alpha(theme.selected_bg, 0x40);
        dot_cell = dot_cell
            .cursor_pointer()
            .hover(move |s| s.bg(rgba(hover_bg)))
            .on_mouse_down(
                MouseButton::Left,
                // stop_propagation keeps the row's own left-click handler from also firing when the dot is clicked.
                cx.listener(move |v, ev: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    v.open_gutter_context_menu(dot_path.clone(), line_ix, ev.position, cx);
                }),
            );
    }
    let row = interactive_gutter_row(
        &line.line,
        theme,
        is_selected,
        review_cell,
        dot_cell.into_any_element(),
    );
    attach_gutter_selection_handlers(row, path.to_owned(), line_ix, cx).into_any_element()
}

struct ContentRowState<'a> {
    ix: usize,
    rendered: &'a DiffRenderRows,
    lines: &'a [WrappedDiffLine],
    path: &'a str,
    selection: Option<DiffSelection>,
    gutter_selection: Option<&'a GutterLineSelection>,
    theme: &'a Theme,
    query: Option<&'a str>,
    advance: Pixels,
    bounds: &'a PanelBoundsSlot,
    extends_selection: bool,
}

fn content_row_at(state: ContentRowState<'_>, cx: &mut Context<RepoWindow>) -> AnyElement {
    let ContentRowState {
        ix,
        rendered,
        lines,
        path,
        selection,
        gutter_selection,
        theme,
        query,
        advance,
        bounds,
        extends_selection,
    } = state;
    match &rendered.rows[ix] {
        DiffRenderRow::Line(w_ix) => {
            let line = &lines[*w_ix];
            if let Some(region) = line.line.context_region {
                return content_row(&line.line, theme, query, None, advance)
                    .debug_selector(move || format!("diff-content-row-{ix}"))
                    .child(context_controls("unified", region, theme, cx))
                    .into_any_element();
            }
            // Shared wrap records use u32; GPUI selection geometry is usize.
            let line_ix = line.line_ix as usize;
            let line_len = line.line_len as usize;
            let col_start = line.col_start as usize;
            let col_end = line.col_end as usize;
            let selection_cols = selection.and_then(|s| {
                if s.side == SbsSide::Unified {
                    s.col_range_for(line_ix, line_len)
                        .and_then(|cols| selection_cols_in_fragment(cols, col_start, col_end))
                } else {
                    None
                }
            });
            let mut row = content_row(&line.line, theme, query, selection_cols, advance)
                .debug_selector(move || format!("diff-content-row-{ix}"));
            if gutter_selection.is_some_and(|s| s.covers(path, line_ix)) {
                row = row.child(content_row_tint(theme));
            }
            if let Some(moved) = line.line.moved {
                row = row.children(moved_label(
                    ("moved-unified", ix),
                    moved,
                    line.line.style,
                    col_start == 0,
                    theme,
                    cx,
                ));
            }
            attach_selection_handlers(
                row,
                SelectionCell {
                    ix: line_ix,
                    side: SbsSide::Unified,
                    advance,
                    col_offset: col_start,
                    bounds: bounds.clone(),
                    extends: extends_selection,
                },
                cx,
            )
            .into_any_element()
        }
        DiffRenderRow::NoteText {
            text,
            is_first,
            is_last,
            ..
        } => {
            let indent_cols = rendered.note_indents.get(&ix).copied().unwrap_or(0);
            let indent = px(CODE_INSET + indent_cols as f32 * f32::from(advance));
            note_content_row(text.clone(), theme, *is_first, *is_last, indent).into_any_element()
        }
    }
}
