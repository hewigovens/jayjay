use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::revset_presets;

use super::super::RepoWindow;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::input::{LineInput, line_input_content};
use crate::ui::primitives::{icon_button, inert_icon_button, text_tooltip};

pub(in super::super) fn revset_filter_panel(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    let input = view.revset_filter.as_ref()?;
    let current_revset = view.vm.read(cx).revset.clone();
    let apply_enabled = input.text() != current_revset.as_ref();

    Some(
        div()
            .id("revset-filter")
            .debug_selector(|| "revset-filter".to_owned())
            .flex()
            .flex_col()
            .gap(px(6.))
            .px(px(12.))
            .py(px(8.))
            .border_b_1()
            .border_color(rgb(t.border))
            .bg(rgb(t.sidebar_bg))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(revset_input(view, input, t, cx))
                    .child(apply_button(apply_enabled, t, cx))
                    .child(reset_button(t, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .flex_wrap()
                    .children(revset_presets().iter().map(|preset| {
                        revset_chip(
                            preset.id.as_str(),
                            preset.label.as_str(),
                            preset.revset.as_str(),
                            current_revset.as_ref() == preset.revset,
                            t,
                            cx,
                        )
                    })),
            )
            .into_any_element(),
    )
}

fn revset_input(
    view: &RepoWindow,
    input: &LineInput,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .id("revset-filter-input")
        .debug_selector(|| "revset-filter-input".to_owned())
        .flex()
        .items_center()
        .flex_1()
        .min_w_0()
        .h(px(28.))
        .px(px(8.))
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.detail_bg))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .cursor_text()
        .track_focus(&view.revset_filter_focus)
        .focus(|style| style.border_color(rgb(t.selected_accent)))
        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
            view.activate_revset_filter(window, cx);
        }))
        .on_key_down(cx.listener(|view, ev, window, cx| {
            if view.handle_revset_filter_key(ev, window, cx) {
                cx.stop_propagation();
            }
        }))
        .child(line_input_content(
            input,
            "Revset expression",
            t,
            Some("revset-filter-caret"),
        ))
        .into_any_element()
}

fn apply_button(enabled: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let button = if enabled {
        icon_button(
            "revset-filter-apply",
            glyph::ARROW_CIRCLE_RIGHT,
            15.,
            24.,
            24.,
            t.fg_dim,
            t,
        )
        .cursor_pointer()
        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
            view.apply_revset_filter(cx);
            view.focus_handle.focus(window, cx);
        }))
    } else {
        inert_icon_button(
            "revset-filter-apply",
            glyph::ARROW_CIRCLE_RIGHT,
            15.,
            24.,
            24.,
            t.fg_faint,
        )
    };
    button
        .debug_selector(|| "revset-filter-apply".to_owned())
        .tooltip(text_tooltip("Apply revset"))
        .into_any_element()
}

fn reset_button(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    icon_button(
        "revset-filter-reset",
        glyph::X_CIRCLE,
        15.,
        24.,
        24.,
        t.fg_faint,
        t,
    )
    .debug_selector(|| "revset-filter-reset".to_owned())
    .tooltip(text_tooltip("Reset to default"))
    .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
        view.reset_revset_filter(cx);
        view.focus_handle.focus(window, cx);
    }))
    .into_any_element()
}

fn revset_chip(
    id: &'static str,
    label: &'static str,
    revset: &'static str,
    active: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let selector = SharedString::from(format!("revset-chip-{id}"));
    let (background, foreground) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    div()
        .id(selector.clone())
        .debug_selector(move || selector.to_string())
        .flex()
        .flex_none()
        .items_center()
        .h(px(24.))
        .px(px(10.))
        .rounded_full()
        .bg(rgb(background))
        .text_size(px(11.))
        .text_color(rgb(foreground))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(cx.listener(move |view, _: &ClickEvent, window, cx| {
            view.select_revset_preset(revset, cx);
            view.focus_handle.focus(window, cx);
        }))
        .child(label)
        .into_any_element()
}
