use gpui::{
    AnyElement, ClipboardItem, Context, Div, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, Stateful, StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{boolean_toggle_button, icon_button};

use super::SettingsView;

pub(super) fn form_section(
    title: impl Into<SharedString>,
    rows: impl IntoIterator<Item = AnyElement>,
    t: &Theme,
) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .gap(px(8.))
        .child(
            div()
                .px(px(12.))
                .text_size(ui_font_size(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg_dim))
                .child(title.into()),
        )
        .child(form_group(rows, t))
}

pub(super) fn form_group(rows: impl IntoIterator<Item = AnyElement>, t: &Theme) -> Div {
    let mut group = div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .px(px(12.))
        .rounded(px(10.))
        .bg(rgb(t.row_alt_bg));
    for (index, row) in rows.into_iter().enumerate() {
        if index > 0 {
            group = group.child(div().flex_none().h(px(1.)).bg(rgb(t.row_border)));
        }
        group = group.child(div().flex_none().w_full().min_w_0().child(row));
    }
    group
}

pub(super) fn row_container() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .min_w_0()
        .min_h(px(40.))
        .gap(px(8.))
}

pub(super) fn detail_row(
    glyph_str: &'static str,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    detail_font_size: f32,
    detail_color: u32,
    t: &Theme,
) -> Div {
    row_container()
        .py(px(5.))
        .child(
            icons::icon(glyph_str, t.scaled_font_size(14.), t.fg_dim)
                .w(px(t.scaled_font_size(16.))),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg))
                .child(label.into()),
        )
        .child(
            div()
                .flex_shrink_1()
                .max_w(gpui::relative(0.55))
                .min_w_0()
                .truncate()
                .code_text(detail_font_size, t)
                .text_color(rgb(detail_color))
                .child(detail.into()),
        )
}

pub(super) fn field_row(
    glyph_str: &'static str,
    label: &'static str,
    value: AnyElement,
    t: &Theme,
) -> AnyElement {
    row_container()
        .min_h(px(40.))
        .py(px(6.))
        .gap(px(6.))
        .child(
            icons::icon(glyph_str, t.scaled_font_size(14.), t.fg_dim)
                .w(px(t.scaled_font_size(16.))),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg))
                .child(label),
        )
        .child(
            div()
                .max_w(gpui::relative(0.6))
                .min_w_0()
                .ml(px(6.))
                .child(value),
        )
        .into_any_element()
}

pub(super) fn stepper(
    id: &'static str,
    label: String,
    value: f32,
    bounds: std::ops::RangeInclusive<f32>,
    adjust: fn(&mut crate::app::config::AppConfig, f32),
    t: &Theme,
) -> AnyElement {
    let step_button = |suffix: &'static str, label: &'static str, sign: f32, enabled: bool| {
        let id = format!("setting-{id}-{suffix}");
        let button = div()
            .id(SharedString::from(id.clone()))
            .debug_selector(move || id.clone())
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .w(px(t.scaled_control_height(28., 14.)))
            .h(px(t.scaled_control_height(28., 14.)))
            .bg(rgb(t.toggle_inactive_bg))
            .text_size(ui_font_size(14.))
            .text_color(rgb(t.fg))
            .child(label);
        if enabled {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgb(t.row_alt_bg)))
                .on_click(move |_: &gpui::ClickEvent, _, cx| {
                    config::update(cx, move |cfg| adjust(cfg, sign));
                })
                .into_any_element()
        } else {
            button.opacity(0.45).into_any_element()
        }
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .child(current_value(&label, t))
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(1.))
                .p(px(1.))
                .rounded_md()
                .overflow_hidden()
                .bg(rgb(t.border))
                .child(step_button("decrease", "−", -1., value > *bounds.start()))
                .child(step_button("increase", "+", 1., value < *bounds.end())),
        )
        .into_any_element()
}

pub(super) fn current_value(value: &str, t: &Theme) -> AnyElement {
    div()
        .max_w(px(360.))
        .min_w_0()
        .truncate()
        .text_size(ui_font_size(12.))
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
    glyph_str: &'static str,
    label: &'static str,
    active: bool,
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

    field_row(glyph_str, label, value, t)
}
