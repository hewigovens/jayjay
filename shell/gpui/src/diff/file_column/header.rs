use gpui::{
    ClickEvent, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::theme::{FONT_META, Theme};
use crate::repo::window::RepoWindow;
use crate::ui::icons::glyph;
use crate::ui::primitives::{icon_button, text_tooltip};

pub(super) struct FileHeaderFilters {
    pub show_review: bool,
    pub hide_reviewed: bool,
    pub active_note_count: usize,
    pub notes_only: bool,
}

pub(super) fn file_column_header(
    reviewed: usize,
    count: usize,
    loading: bool,
    filters: &FileHeaderFilters,
    tree_mode: bool,
    cx: &mut Context<RepoWindow>,
    t: &Theme,
) -> impl IntoElement {
    let &FileHeaderFilters {
        show_review,
        hide_reviewed,
        active_note_count,
        notes_only,
    } = filters;
    // Mirrors SwiftUI's `fileCountLabel`: always the plain file count, never folding in review state (the reviewed/total badge is its own element to the right).
    let label = if loading {
        String::from("Loading…")
    } else if count == 0 {
        String::from("0 files")
    } else {
        format!("{count} files")
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
        .h(px(40.))
        .px(px(12.))
        .bg(rgb(t.header_bg))
        .debug_selector(|| "file-column-header".to_owned())
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(label)),
        )
        .child(div().flex_1());

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
                .mr(px(6.))
                .rounded_sm()
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
                .mr(px(6.))
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
            .mr(px(6.))
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
            .mr(px(6.))
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
    row.child(
        icon_button("toggle-file-tree", tree_glyph, 13., 24., 22., fg, t)
            .debug_selector(|| "toggle-file-tree".to_owned())
            .bg(rgb(bg))
            .tooltip(text_tooltip(tree_help))
            .on_click(|_, _, cx| config::update(cx, |c| c.diff.tree_file_list ^= true)),
    )
}
