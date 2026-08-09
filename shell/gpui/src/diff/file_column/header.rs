use gpui::{
    AnyElement, ClickEvent, Context, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::{FONT_META, Theme};
use crate::app::{config, fonts};
use crate::repo::window::RepoWindow;
use crate::ui::icons::glyph;
use crate::ui::input::{LineInput, line_input_content};
use crate::ui::primitives::{icon_button, text_tooltip};

pub(super) struct FileHeaderState {
    pub reviewed: usize,
    pub count: usize,
    pub visible_count: usize,
    pub loading: bool,
    pub show_review: bool,
    pub hide_reviewed: bool,
    pub active_note_count: usize,
    pub notes_only: bool,
    pub file_filter_active: bool,
    pub file_filter_has_query: bool,
    pub tree_mode: bool,
}

pub(super) fn file_column_header(
    state: FileHeaderState,
    cx: &mut Context<RepoWindow>,
    t: &Theme,
) -> impl IntoElement {
    let FileHeaderState {
        reviewed,
        count,
        visible_count,
        loading,
        show_review,
        hide_reviewed,
        active_note_count,
        notes_only,
        file_filter_active,
        file_filter_has_query,
        tree_mode,
    } = state;
    // Mirrors SwiftUI's file-column count: filename filtering shows the visible/total pair, while the reviewed/total badge remains separate.
    let label = if loading {
        String::from("Loading…")
    } else if file_filter_has_query {
        format!("{visible_count} of {count} files")
    } else if visible_count == 0 {
        String::from("0 files")
    } else {
        format!("{visible_count} files")
    };
    let (tree_glyph, tree_help) = if tree_mode {
        (glyph::LIST_TREE, "Showing files as a tree")
    } else {
        (glyph::LIST, "Showing files as a flat list")
    };
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .h(px(40.))
        .px(px(12.))
        .bg(rgb(t.header_bg))
        .debug_selector(|| "file-column-header".to_owned())
        .child(
            div()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(label)),
        )
        .child(div().flex_1());
    if !file_filter_active {
        row = row.border_b_1().border_color(rgb(t.border));
    }

    if show_review && active_note_count > 0 {
        let (bg, fg) = if notes_only {
            (t.toggle_active_bg, t.toggle_active_fg)
        } else {
            (t.toggle_inactive_bg, t.toggle_inactive_fg)
        };
        row = row.child(
            div()
                .id("file-notes-only")
                .debug_selector(|| "file-notes-only".to_owned())
                .flex()
                .flex_row()
                .items_center()
                .gap(px(3.))
                .h(px(22.))
                .px(px(6.))
                .rounded_md()
                .bg(rgb(bg))
                .text_size(px(10.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(fg))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(t.row_alt_bg)))
                .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
                    view.toggle_notes_only_files(cx);
                }))
                .tooltip(text_tooltip(if notes_only {
                    "Showing only files with review notes"
                } else {
                    "Show only files with review notes"
                }))
                .child(SharedString::from(format!("\u{25cf} {active_note_count}"))),
        );
    }

    if show_review && reviewed > 0 {
        // SwiftUI parity (`FileColumn.swift`): the reviewed/total count and quick-split button target the currently reviewed (checked) files, not the row multi-selection.
        row = row.child(
            div()
                .id("file-reviewed-count")
                .debug_selector(|| "file-reviewed-count".to_owned())
                .text_size(px(10.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.fg_dim))
                .tooltip(text_tooltip(format!(
                    "{reviewed} of {count} files reviewed"
                )))
                .child(SharedString::from(format!("{reviewed}/{count}"))),
        );
        row = row.child(
            icon_button(
                "file-split-reviewed",
                glyph::GIT_BRANCH,
                11.,
                24.,
                22.,
                t.fg_dim,
                t,
            )
            .debug_selector(|| "file-split-reviewed".to_owned())
            .tooltip(text_tooltip(format!(
                "Split {reviewed} checked files to a new change"
            )))
            .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
                view.open_reviewed_files_split_modal(cx);
            })),
        );
        let (bg, fg) = if hide_reviewed {
            (t.toggle_active_bg, t.toggle_active_fg)
        } else {
            (t.toggle_inactive_bg, t.toggle_inactive_fg)
        };
        row = row.child(
            icon_button(
                "file-hide-reviewed",
                if hide_reviewed {
                    glyph::EYE_OFF
                } else {
                    glyph::EYE
                },
                13.,
                24.,
                22.,
                fg,
                t,
            )
            .debug_selector(|| "file-hide-reviewed".to_owned())
            .bg(rgb(bg))
            .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
                view.toggle_hide_reviewed_files(cx);
            })),
        );
    }

    let (bg, fg) = if tree_mode {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    row = row.child(
        icon_button("toggle-file-tree", tree_glyph, 13., 24., 22., fg, t)
            .debug_selector(|| "toggle-file-tree".to_owned())
            .bg(rgb(bg))
            .tooltip(text_tooltip(tree_help))
            .on_click(|_, _, cx| config::update(cx, |c| c.diff.tree_file_list ^= true)),
    );

    let (bg, fg) = if file_filter_active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    row.child(
        icon_button("toggle-file-filter", glyph::SEARCH, 13., 24., 22., fg, t)
            .debug_selector(|| "toggle-file-filter".to_owned())
            .bg(rgb(bg))
            .tooltip(text_tooltip("Filter files"))
            .on_click(cx.listener(|view, _event: &ClickEvent, window, cx| {
                view.toggle_file_filter(window, cx);
            })),
    )
}

pub(super) fn file_filter_bar(
    input: &LineInput,
    focus: &FocusHandle,
    cx: &mut Context<RepoWindow>,
    t: &Theme,
) -> AnyElement {
    div()
        .debug_selector(|| "file-filter-bar".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(10.))
        .pb(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .id("file-filter-input")
                .debug_selector(|| "file-filter-input".to_owned())
                .flex()
                .items_center()
                .flex_1()
                .min_w_0()
                .h(px(26.))
                .px(px(8.))
                .rounded_md()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.detail_bg))
                .font_family(fonts::mono())
                .text_size(px(11.))
                .cursor_text()
                .track_focus(focus)
                .focus(|style| style.border_color(rgb(t.selected_accent)))
                .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                    view.activate_file_filter(window, cx);
                }))
                .on_key_down(cx.listener(|view, event, window, cx| {
                    if view.handle_file_filter_key(event, window, cx) {
                        cx.stop_propagation();
                    }
                }))
                .child(line_input_content(
                    input,
                    "Filter files",
                    t,
                    Some("file-filter-caret"),
                )),
        )
        .child(
            icon_button(
                "file-filter-close",
                glyph::X_CIRCLE,
                13.,
                22.,
                22.,
                t.fg_faint,
                t,
            )
            .debug_selector(|| "file-filter-close".to_owned())
            .tooltip(text_tooltip("Close file filter"))
            .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                view.close_file_filter(cx);
                view.focus_handle.focus(window, cx);
            })),
        )
        .into_any_element()
}
