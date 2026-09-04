use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Pixels, Styled, UniformListScrollHandle, div, px, rgba,
    uniform_list,
};
use jayjay_core::diff::{FileDiff, WrappedDiffLine};
use jayjay_review::ReviewNoteStatus;

use super::context_controls::context_controls;
use super::gutter_mouse::attach_gutter_selection_handlers;
use super::mouse::attach_selection_handlers;
use super::rows::{DiffRenderRow, DiffRenderRows};
use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::diff::line::{
    content_row, content_row_tint, interactive_gutter_column, interactive_gutter_row,
    line_bg_color, note_content_row, note_dot_cell, note_gutter_row,
};
use crate::diff::wrap::{selection_cols_in_fragment, wrap_cols_from_bounds};
use crate::diff::{DiffSelection, GutterLineSelection, SbsSide, bounds_capture};
use crate::repo::window::{DiffWrapCacheSlot, PanelBoundsSlot, RepoWindow};
use crate::ui::primitives::no_scrollbar_gutter;
use crate::ui::scrollbar::vertical_uniform_scrollbar;

pub(super) struct UnifiedBodyState<'a> {
    pub(super) file_diff: &'a Arc<FileDiff>,
    pub(super) theme: Theme,
    pub(super) query: Option<String>,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) bounds: PanelBoundsSlot,
    pub(super) wrap_cache: &'a DiffWrapCacheSlot,
    pub(super) notes: &'a [ReviewNoteStatus],
}

pub(super) fn unified_body(
    state: UnifiedBodyState<'_>,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let UnifiedBodyState {
        file_diff,
        theme,
        query,
        scroll,
        bounds,
        wrap_cache,
        notes,
    } = state;
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let advance = fonts::mono_advance(cx, px(theme.font_size));
    let wrap_cols = wrap_cols_from_bounds(bounds.get(), advance);
    let lines = wrap_cache.borrow_mut().unified(file_diff, wrap_cols);
    // Both lists size off this shared, interleaved row list — never off lines.len() — so a NoteText row shifts gutter and content lists in lockstep.
    let rendered = wrap_cache.borrow_mut().rows(file_diff, wrap_cols, notes);
    let count = rendered.rows.len();

    let gutter_lines = lines.clone();
    let gutter_rendered = rendered.clone();
    let gutter_theme = theme.clone();
    let gutter_path = file_diff.path.clone();
    let gutter = uniform_list(
        "diff-gutter",
        count,
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
                        },
                        cx,
                    )
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);

    let content_lines = lines;
    let content_rendered = rendered;
    let content_theme = theme.clone();
    let content_query = query;
    let content_bounds = bounds.clone();
    let content_path = file_diff.path.clone();
    let content = uniform_list(
        "diff-content",
        count,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let sel = view.diff.selection;
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
                        },
                        cx,
                    )
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
            interactive_gutter_column(theme.as_ref())
                .debug_selector(|| "diff-gutter".to_owned())
                .child(no_scrollbar_gutter(gutter).h_full()),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w_0()
                .h_full()
                .child(bounds_capture(bounds.clone()))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|v, _: &MouseUpEvent, _, cx| {
                        v.finish_diff_selection(cx);
                    }),
                )
                .child(no_scrollbar_gutter(content).h_full())
                .child(vertical_uniform_scrollbar(
                    scroll,
                    bounds,
                    px(count as f32 * theme.code_line_height()),
                    theme.as_ref(),
                    cx,
                )),
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
}

fn gutter_row_at(state: GutterRowState<'_>, cx: &mut Context<RepoWindow>) -> AnyElement {
    let GutterRowState {
        ix,
        rendered,
        lines,
        path,
        selection,
        theme,
    } = state;
    let DiffRenderRow::Line(w_ix) = &rendered.rows[ix] else {
        return note_gutter_row(theme).into_any_element();
    };
    let w_ix = *w_ix;
    let line = &lines[w_ix];
    if line.line.context_region.is_some() {
        return interactive_gutter_row(
            &line.line,
            theme,
            false,
            note_dot_cell(None, theme, theme.diff_separator_bg).into_any_element(),
        )
        .into_any_element();
    }
    let line_ix = line.line_ix as usize;
    let is_selected = selection.is_some_and(|sel| sel.covers(path, line_ix));
    let dot = rendered.dots.get(&w_ix).copied();
    let line_bg = line_bg_color(line.line.style, line.line.conflict_kind, theme);
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
    let row = interactive_gutter_row(&line.line, theme, is_selected, dot_cell.into_any_element());
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
            attach_selection_handlers(
                row,
                line_ix,
                SbsSide::Unified,
                advance,
                col_start,
                bounds.clone(),
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
            let indent = px(indent_cols as f32 * f32::from(advance));
            note_content_row(text.clone(), theme, *is_first, *is_last, indent).into_any_element()
        }
    }
}
