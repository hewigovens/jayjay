use crate::app::config::{self, AppConfig};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::logo::Logo;
use crate::ui::primitives::boolean_toggle_button;
use gpui::{
    ClickEvent, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

const APP_NAME: &str = "JayJay";
const TAGLINE: &str = "A native GUI for Jujutsu";
const SPONSOR_URL: &str = "https://github.com/sponsors/hewigovens";
const GITHUB_URL: &str = "https://github.com/hewigovens/jayjay";

pub(super) fn about_section(cfg: &AppConfig, logo: &Logo, t: &Theme) -> impl IntoElement {
    let version = format!("Version {} (GPUI Alpha)", env!("CARGO_PKG_VERSION"));

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .pt(px(8.))
        .child(logo.image(72.))
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
                .child(TAGLINE),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(version)),
        )
        .child(telemetry_toggle(cfg.telemetry.enabled, t))
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .pt(px(8.))
                .child(link_button(
                    "tb-sponsor",
                    glyph::SPARKLE,
                    "Sponsor",
                    SPONSOR_URL,
                    t,
                ))
                .child(link_button(
                    "tb-github",
                    glyph::ARROW_CIRCLE_RIGHT,
                    "Star on GitHub",
                    GITHUB_URL,
                    t,
                )),
        )
}

fn telemetry_toggle(active: bool, t: &Theme) -> impl IntoElement {
    let value = boolean_toggle_button(
        SharedString::from("setting-about-telemetry"),
        active,
        t,
        move |_, _, cx| {
            let enabled = !active;
            config::update(cx, |c| c.telemetry.enabled = enabled);
            crate::app::telemetry::maybe_ping(enabled);
        },
    );

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.))
        .pt(px(2.))
        .max_w(px(560.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(14.))
                .flex_wrap()
                .child(
                    div()
                        .text_size(ui_font_size(12.))
                        .text_color(rgb(t.fg))
                        .child("Share anonymous build and OS stats"),
                )
                .child(value),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_faint))
                .text_center()
                .child("No repository, file, or command data is sent."),
        )
}

fn link_button(
    id: &'static str,
    glyph_str: &'static str,
    label: &'static str,
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
