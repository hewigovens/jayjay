use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

use super::FindState;
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::input::{LineInput, line_input_content};

pub(super) fn render_find_bar(
    query: &LineInput,
    find: &FindState<'_>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (match_count, match_current) = (find.match_count, find.match_current);
    let count_label = if query.is_empty() {
        String::new()
    } else if match_count == 0 {
        String::from("No matches")
    } else {
        format!("{} of {}", match_current + 1, match_count)
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(glyph::SEARCH, 12., t.fg_dim))
        .child(search_input(query, find.ime_focus.clone(), t, cx))
        .child(
            div()
                .flex_none()
                .text_size(ui_font_size(10.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(count_label)),
        )
        .child(nav_controls(match_count > 0, t, cx))
        .into_any_element()
}

fn search_input(
    query: &LineInput,
    ime_focus: Option<gpui::FocusHandle>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut input = div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .flex_1()
        .min_w_0()
        .code_text(12.);

    input = input
        .child(LineInput::ime_layer(
            cx.entity(),
            ime_focus,
            RepoWindow::find_input,
            RepoWindow::find_query_edited,
        ))
        .child(line_input_content(query, "Type to find...", t, None));

    input.into_any_element()
}

fn nav_controls(enabled: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .child(nav_button("<", "Previous match", true, enabled, t, cx))
        .child(nav_button(">", "Next match", false, enabled, t, cx))
        .child(done_button(t, cx))
        .into_any_element()
}

fn nav_button(
    symbol: &'static str,
    label: &'static str,
    previous: bool,
    enabled: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let fg = if enabled { t.fg_dim } else { t.fg_faint };
    let hover_bg = t.row_alt_bg;
    let mut button = div()
        .id(SharedString::from(format!("find-nav-{label}")))
        .flex()
        .items_center()
        .justify_center()
        .w(px(t.scaled_control_height(20., 11.)))
        .h(px(t.scaled_control_height(20., 11.)))
        .rounded_md()
        .code_text(11.)
        .text_color(rgb(fg))
        .child(symbol);

    if enabled {
        button = button
            .cursor_pointer()
            .hover(move |s| s.bg(rgb(hover_bg)))
            .on_click(
                cx.listener(move |view, _ev: &ClickEvent, _w: &mut Window, cx| {
                    view.find_advance(previous, cx);
                }),
            );
    }

    button.into_any_element()
}

fn done_button(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let hover_bg = t.row_alt_bg;
    div()
        .id("find-done")
        .flex()
        .items_center()
        .justify_center()
        .h(px(t.scaled_control_height(20., 11.)))
        .px(px(6.))
        .rounded_md()
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .child("Done")
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover_bg)))
        .on_click(cx.listener(|view, _ev: &ClickEvent, _w: &mut Window, cx| {
            view.close_find(cx);
        }))
        .into_any_element()
}
