use gpui::{
    ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::theme::{FONT_META, Theme};
use crate::repo::window::RepoWindow;
use crate::ui::icons::glyph;
use crate::ui::primitives::{icon_button, text_tooltip};

#[allow(clippy::too_many_arguments)]
pub(super) fn file_column_header(
    reviewed: usize,
    count: usize,
    loading: bool,
    show_review: bool,
    hide_reviewed: bool,
    tree_mode: bool,
    cx: &mut Context<RepoWindow>,
    t: &Theme,
) -> impl IntoElement {
    let label = if loading {
        String::from("Loading…")
    } else if count == 0 {
        String::from("0 files")
    } else if show_review {
        format!("{reviewed} / {count} reviewed")
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

    if show_review && reviewed > 0 {
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
