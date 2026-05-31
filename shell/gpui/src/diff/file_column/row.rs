use gpui::{
    AnyElement, App, ClickEvent, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn row_bg(is_selected: bool, _ix: usize, t: &Theme) -> u32 {
    if is_selected {
        t.selected_bg
    } else {
        t.sidebar_bg
    }
}

pub(super) fn review_checkbox<FRev>(
    id: (&'static str, usize),
    reviewed: bool,
    t: &Theme,
    on_click: FRev,
) -> AnyElement
where
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let (bg, fg, border) = if reviewed {
        (t.toggle_active_bg, t.toggle_active_fg, t.toggle_active_bg)
    } else {
        (t.sidebar_bg, t.fg_faint, t.border)
    };
    let mut box_div = div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.))
        .h(px(14.))
        .rounded_sm()
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(bg))
        .cursor_pointer()
        .on_click(move |ev, w, cx| {
            cx.stop_propagation();
            on_click(ev, w, cx);
        });
    if reviewed {
        box_div = box_div.child(icons::icon(glyph::CHECK, 10., fg));
    }
    box_div.into_any_element()
}

pub(super) fn status_dot(hunk: &DiffHunk, t: &Theme) -> impl IntoElement {
    let color = match hunk.hunk_type {
        HunkType::Added => t.tag_added_fg,
        HunkType::Removed => t.tag_removed_fg,
        HunkType::Modified => t.tag_modified_fg,
        HunkType::Renamed => t.tag_renamed_fg,
    };
    div()
        .flex_none()
        .w(px(8.))
        .h(px(8.))
        .rounded_full()
        .bg(rgb(color))
}
