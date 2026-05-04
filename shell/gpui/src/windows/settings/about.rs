use gpui::{
    ClickEvent, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

const APP_NAME: &str = "JayJay";
const TAGLINE: &str = "A native GUI for Jujutsu";
const SPONSOR_URL: &str = "https://github.com/sponsors/hewigovens";
const GITHUB_URL: &str = "https://github.com/hewigovens/jayjay";

pub(super) fn about_section(t: &Theme) -> impl IntoElement {
    let version = format!("Version {} (GPUI Alpha)", env!("CARGO_PKG_VERSION"));

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .pt(px(8.))
        .child(app_icon(t))
        .child(
            div()
                .text_size(px(20.))
                .text_color(rgb(t.fg))
                .child(APP_NAME),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(t.fg_dim))
                .child(TAGLINE),
        )
        .child(
            div()
                .text_size(px(11.))
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

fn app_icon(t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(72.))
        .h(px(72.))
        .rounded_full()
        .bg(rgb(t.toggle_active_bg))
        .child(icons::icon(glyph::GIT_BRANCH, 36., t.toggle_active_fg))
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
        .text_size(px(12.))
        .text_color(rgb(t.toggle_inactive_fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(move |_: &ClickEvent, _, cx| {
            cx.open_url(url);
        })
        .child(icons::icon(glyph_str, 12., t.toggle_inactive_fg))
        .child(label)
}
