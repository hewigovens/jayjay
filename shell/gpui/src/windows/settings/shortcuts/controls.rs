use super::view::ShortcutSettings;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;
use gpui::{
    Context, Div, InteractiveElement, ParentElement, Role, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::rc::Rc;

pub(super) type Action = dyn Fn(&mut ShortcutSettings, &mut Window, &mut Context<ShortcutSettings>);

impl ShortcutSettings {
    pub(super) fn button(
        id: impl Into<SharedString>,
        label: String,
        t: &Theme,
        cx: &Context<Self>,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        Self::control(id.into(), label.clone(), t, cx, Rc::new(action))
            .px(px(8.))
            .py(px(5.))
            .text_size(ui_font_size(12.))
            .child(label)
    }

    pub(super) fn icon_button(
        id: impl Into<SharedString>,
        glyph_str: &'static str,
        aria_label: String,
        t: &Theme,
        cx: &Context<Self>,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        Self::control(id.into(), aria_label, t, cx, Rc::new(action))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(20.))
            .border_color(gpui::transparent_black())
            .child(icons::icon(glyph_str, 12., t.fg_dim))
    }

    pub(super) fn control(
        id: SharedString,
        aria_label: String,
        t: &Theme,
        cx: &Context<Self>,
        action: Rc<Action>,
    ) -> Stateful<Div> {
        let selector = id.to_string();
        let keyboard_action = action.clone();
        div()
            .id(id)
            .debug_selector(move || selector.clone())
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(aria_label)
            .rounded_sm()
            .border_1()
            .border_color(rgb(t.border))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(t.selected_bg)))
            .focus(|s| s.border_color(rgb(t.selected_accent)))
            .on_click(cx.listener(move |this, _, window, cx| action(this, window, cx)))
            .on_key_down(
                cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space")
                        && !event.keystroke.modifiers.modified()
                    {
                        keyboard_action(this, window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
    }
}
