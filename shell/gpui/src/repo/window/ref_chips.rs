use gpui::{ClipboardItem, Context, Div, SharedString, Stateful, StatefulInteractiveElement};

use super::RepoWindow;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::primitives::{capsule, icon_button, icon_chip};

/// A bookmark chip; conflicted bookmarks get the warning treatment and `icon` adds the bookmark glyph the DAG rows show.
pub(crate) fn bookmark_chip(
    name: SharedString,
    conflicted: bool,
    icon: bool,
    font_size: f32,
    t: &Theme,
) -> Div {
    if conflicted {
        icon_chip(
            glyph::WARNING,
            name,
            t.tag_divergent_bg,
            t.tag_divergent_fg,
            t.tag_divergent_fg,
            font_size,
        )
    } else if icon {
        icon_chip(
            glyph::BOOKMARK,
            name,
            t.tag_bookmark_bg,
            t.tag_bookmark_fg,
            t.tag_bookmark_icon,
            font_size,
        )
    } else {
        capsule(name, t.tag_bookmark_bg, t.tag_bookmark_fg, font_size)
    }
}

pub(crate) fn tag_chip(name: SharedString, font_size: f32, t: &Theme) -> Div {
    icon_chip(
        glyph::TAG,
        name,
        t.tag_tag_bg,
        t.tag_tag_fg,
        t.tag_tag_icon,
        font_size,
    )
}

/// Copies `value` and shows a check for a moment through `RepoWindow::mark_copied`, keyed by `id`.
pub(crate) fn copy_feedback_button(
    id: SharedString,
    value: String,
    just_copied: bool,
    idle_color: u32,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Stateful<Div> {
    let (glyph_str, color) = if just_copied {
        (glyph::CHECK, t.success_fg)
    } else {
        (glyph::COPY, idle_color)
    };
    let element_id = SharedString::from(format!("copy-{id}"));
    icon_button(element_id, glyph_str, 12., 20., 20., color, t).on_click(cx.listener(
        move |view, _, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
            view.mark_copied(id.clone(), cx);
        },
    ))
}
