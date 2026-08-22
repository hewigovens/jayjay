use std::path::Path;

use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::app::{config, repositories};
use crate::ui::icons;

pub(super) fn repository_actions(
    path: String,
    pinned: bool,
    pin_id: String,
    remove_id: Option<String>,
    t: &Theme,
) -> Vec<AnyElement> {
    let pin_path = path.clone();
    let mut actions = vec![
        row_action(
            SharedString::from(pin_id),
            if pinned {
                icons::glyph::PIN_OFF
            } else {
                icons::glyph::PIN
            },
            t,
        )
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            repositories::set_pinned(cx, Path::new(&pin_path), !pinned);
        })
        .into_any_element(),
    ];
    if let Some(remove_id) = remove_id {
        actions.push(
            row_action(SharedString::from(remove_id), icons::glyph::X_CIRCLE, t)
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    config::update(cx, |cfg| cfg.remove_recent_repo(&path));
                })
                .into_any_element(),
        );
    }
    actions
}

fn row_action(id: SharedString, glyph: &'static str, t: &Theme) -> gpui::Stateful<Div> {
    div()
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .flex()
        .items_center()
        .justify_center()
        .w(px(24.))
        .h(px(24.))
        .rounded_full()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.selected_bg)))
        .child(icons::icon(glyph, 14., t.fg_faint))
}
