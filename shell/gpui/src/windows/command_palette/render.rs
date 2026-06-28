use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::actions::{ACTIONS, PaletteAction};
use super::state::CommandPalette;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::input::{LineInput, line_input_content};
use crate::ui::primitives::icon_label;

pub(super) fn query_box(query: &LineInput, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(10.))
        .text_size(px(14.))
        .child(icons::icon(glyph::SEARCH, 14., t.fg_dim))
        .child(line_input_content(
            query,
            "Search commands, type `jj status`, or use `!status`",
            t,
            Some("command-palette-caret"),
        ))
}

pub(super) fn divider(t: &Theme) -> impl IntoElement {
    div().h(px(1.)).w_full().bg(rgb(t.border))
}

pub(super) fn action_list(
    visible: &[usize],
    selected: usize,
    t: &Theme,
    cx: &mut Context<CommandPalette>,
) -> AnyElement {
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
        col = col.child(action_row(*action_ix, action, vis_ix == selected, t, cx));
    }
    col.into_any_element()
}

fn action_row(
    action_ix: usize,
    action: &'static PaletteAction,
    is_selected: bool,
    t: &Theme,
    cx: &mut Context<CommandPalette>,
) -> impl IntoElement {
    let selector = action_selector(action.name);
    let label = action.display_name(cx);
    let (bg, fg, glyph_color) = if is_selected {
        (t.selected_bg, t.fg, t.toggle_active_fg)
    } else {
        (t.detail_bg, t.fg, t.fg_dim)
    };
    div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector.clone())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .px(px(14.))
        .py(px(7.))
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(13.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.selected_bg)))
        .on_click(cx.listener(move |palette, _: &ClickEvent, window, cx| {
            palette.dispatch_action(action_ix, window, cx);
        }))
        .child(icon_label(
            action.glyph_str,
            SharedString::from(label),
            14.,
            glyph_color,
        ))
}

fn action_selector(name: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    format!("command-palette-action-{slug}")
}
