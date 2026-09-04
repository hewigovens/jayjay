use gpui::{
    FontWeight, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::logo::Logo;
use crate::ui::primitives::button;

pub(super) fn header(logo: &Logo, t: &Theme) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .child(logo.image(80.))
        .child(
            div()
                .text_size(ui_font_size(28.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(t.fg))
                .child("JayJay"),
        )
        .child(
            div()
                .text_size(ui_font_size(14.))
                .text_color(rgb(t.fg_dim))
                .child("A native GUI for Jujutsu"),
        )
        .child(
            button("repo-list-open", "Open Repository...", t, true)
                .debug_selector(|| "repo-list-open".to_owned())
                .on_click(|_, _, cx| crate::app::menus::prompt_open_repository(cx)),
        )
}
