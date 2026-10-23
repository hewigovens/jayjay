use crate::app::links::{REPOSITORY_URL, SPONSOR_URL};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::logo::Logo;
use gpui::{
    ClickEvent, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_i18n::settings;

const APP_NAME: &str = "JayJay";

pub(super) fn about_section(logo: &Logo, t: &Theme) -> impl IntoElement {
    let version = settings::version(concat!(env!("CARGO_PKG_VERSION"), " (GPUI)"));

    div()
        .debug_selector(|| "settings-about-section".to_owned())
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .pt(px(8.))
        .child(logo.app_icon(72., t))
        .child(
            div()
                .text_size(ui_font_size(20.))
                .text_color(rgb(t.fg))
                .child(APP_NAME),
        )
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_dim))
                .child(settings::tagline()),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(version)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .pt(px(8.))
                .child(link_button(
                    "tb-sponsor",
                    glyph::SPARKLE,
                    settings::sponsor(),
                    SPONSOR_URL,
                    t,
                ))
                .child(link_button(
                    "tb-github",
                    glyph::MARK_GITHUB,
                    settings::star_on_github(),
                    REPOSITORY_URL,
                    t,
                )),
        )
}

fn link_button(
    id: &'static str,
    glyph_str: &'static str,
    label: String,
    url: &'static str,
    t: &Theme,
) -> impl IntoElement {
    div()
        .id(SharedString::from(id))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(14.))
        .py(px(6.))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.toggle_inactive_fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(move |_: &ClickEvent, _, cx| {
            crate::app::links::open_url(cx, url);
        })
        .child(icons::icon(glyph_str, 12., t.toggle_inactive_fg))
        .child(label)
}
