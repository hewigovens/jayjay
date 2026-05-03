use gpui::{
    AnyElement, ClickEvent, ClipboardItem, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::shared::section_title;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn config_section(t: &Theme) -> AnyElement {
    let path = crate::app::config::AppConfig::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unknown — could not resolve project config dir)".to_owned());
    let path_for_copy = path.clone();

    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(section_title("Config", t))
        .child(div().text_size(px(12.)).text_color(rgb(t.fg_dim)).child(
            "JayJay stores its persistent settings as a TOML file at the path below. \
                     Edit it manually if you prefer; changes load on next launch.",
        ))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .py(px(8.))
                .rounded_md()
                .bg(rgb(t.toggle_inactive_bg))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .font_family(crate::app::fonts::mono())
                        .text_size(px(11.))
                        .text_color(rgb(t.fg))
                        .child(SharedString::from(path)),
                )
                .child(copy_button(path_for_copy, t)),
        )
        .into_any_element()
}

fn copy_button(value: String, t: &Theme) -> AnyElement {
    div()
        .id(SharedString::from("config-copy-path"))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(24.))
        .h(px(20.))
        .rounded_sm()
        .cursor_pointer()
        .text_color(rgb(t.fg_faint))
        .on_click(move |_: &ClickEvent, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
        })
        .child(icons::icon(glyph::COPY, 12., t.fg_faint))
        .into_any_element()
}
