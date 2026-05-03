use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use super::actions::{ACTIONS, PaletteAction};
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn query_box(query: &str, t: &Theme) -> AnyElement {
    let display = if query.is_empty() {
        SharedString::from("Type to search…")
    } else {
        SharedString::from(query.to_owned())
    };
    let color = if query.is_empty() { t.fg_faint } else { t.fg };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(10.))
        .text_size(px(14.))
        .text_color(rgb(color))
        .child(icons::icon(glyph::SEARCH, 14., t.fg_dim))
        .child(display)
        .into_any_element()
}

pub(super) fn divider(t: &Theme) -> AnyElement {
    div()
        .h(px(1.))
        .w_full()
        .bg(rgb(t.border))
        .into_any_element()
}

pub(super) fn action_list(visible: &[usize], selected: usize, t: &Theme) -> AnyElement {
    if visible.is_empty() {
        return div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("No matches")
            .into_any_element();
    }
    let mut col = div().flex().flex_col().flex_1().min_h_0().py(px(4.));
    for (vis_ix, action_ix) in visible.iter().enumerate() {
        let action = &ACTIONS[*action_ix];
        col = col.child(action_row(action, vis_ix == selected, t));
    }
    col.into_any_element()
}

fn action_row(action: &'static PaletteAction, is_selected: bool, t: &Theme) -> AnyElement {
    let (bg, fg, glyph_color) = if is_selected {
        (t.selected_bg, t.fg, t.toggle_active_fg)
    } else {
        (t.detail_bg, t.fg, t.fg_dim)
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .px(px(14.))
        .py(px(7.))
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(13.))
        .child(icons::icon(action.glyph_str, 14., glyph_color))
        .child(SharedString::from(action.name))
        .into_any_element()
}
