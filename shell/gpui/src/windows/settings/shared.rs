use gpui::{
    AnyElement, ClipboardItem, Context, Div, InteractiveElement, IntoElement, ParentElement,
    SharedString, Stateful, StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{boolean_toggle_button, icon_button};

use super::SettingsView;

pub(super) fn section_title(text: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .w_full()
        .text_size(px(18.))
        .text_color(rgb(t.fg))
        .pb(px(4.))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(text)
}

pub(super) fn subsection_title(text: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .w_full()
        .pt(px(4.))
        .text_size(px(11.))
        .text_color(rgb(t.fg_faint))
        .child(text)
}

pub(super) fn row_container(t: &Theme) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(8.))
        .px(px(8.))
        .rounded_sm()
        .bg(rgb(t.row_alt_bg))
}

pub(super) fn detail_row(
    glyph_str: &'static str,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    detail_font_size: f32,
    detail_color: u32,
    t: &Theme,
) -> Div {
    row_container(t)
        .py(px(5.))
        .child(icons::icon(glyph_str, 14., t.fg_dim))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.))
                .text_color(rgb(t.fg))
                .child(label.into()),
        )
        .child(
            div()
                .flex_none()
                .max_w(px(360.))
                .min_w_0()
                .truncate()
                .font_family(crate::app::fonts::mono())
                .text_size(px(detail_font_size))
                .text_color(rgb(detail_color))
                .child(detail.into()),
        )
}

pub(super) fn field_row(
    label: &'static str,
    value: AnyElement,
    hint: &'static str,
    t: &Theme,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(4.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .gap(px(12.))
                .child(div().text_size(px(12.)).text_color(rgb(t.fg)).child(label))
                .child(value),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child(hint),
        )
        .into_any_element()
}

pub(super) fn current_value(value: &str, t: &Theme) -> AnyElement {
    div()
        .max_w(px(360.))
        .min_w_0()
        .truncate()
        .text_size(px(12.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(value.to_owned()))
        .into_any_element()
}

pub(super) fn feedback_copy_icon_button(
    id: &'static str,
    value: impl Into<String>,
    copied: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> Stateful<Div> {
    let selector = copy_feedback_selector(id, copied);
    copy_action(
        icon_button(
            id,
            if copied { glyph::CHECK } else { glyph::COPY },
            12.,
            24.,
            20.,
            if copied { t.success_fg } else { t.fg_faint },
            t,
        )
        .debug_selector(move || selector.clone()),
        id,
        value.into(),
        cx,
    )
}

fn copy_action(
    element: Stateful<Div>,
    id: &'static str,
    value: String,
    cx: &mut Context<SettingsView>,
) -> Stateful<Div> {
    element.on_click(cx.listener(move |view, _, _, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
        view.mark_copied(id.into(), cx);
    }))
}

fn copy_feedback_selector(id: &str, copied: bool) -> String {
    if copied {
        format!("{id}-copied")
    } else {
        id.to_owned()
    }
}

pub(super) fn toggle_field(
    label: &'static str,
    active: bool,
    hint: &'static str,
    mutate: fn(&mut crate::app::config::AppConfig),
    id: &'static str,
    t: &Theme,
) -> AnyElement {
    let value = boolean_toggle_button(
        SharedString::from(format!("setting-{id}")),
        active,
        t,
        move |_, _, cx| {
            config::update(cx, mutate);
        },
    );

    field_row(label, value, hint, t)
}
