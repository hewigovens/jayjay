use gpui::{IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::config;
use crate::app::theme::{FONT_META, Theme};
use crate::ui::icons::glyph;
use crate::ui::primitives::toggle_button;

pub(super) fn file_column_header(
    reviewed: usize,
    count: usize,
    loading: bool,
    show_review: bool,
    tree_mode: bool,
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
    let (tree_glyph, tree_label) = if tree_mode {
        (glyph::FOLDER, "Tree")
    } else {
        (glyph::ROWS, "Flat")
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(label)),
        )
        .child(div().flex_1())
        .child(toggle_button(
            tree_glyph,
            tree_label,
            "file-tree",
            tree_mode,
            t,
            |_, _, cx| config::update(cx, |c| c.diff.tree_file_list ^= true),
        ))
}
