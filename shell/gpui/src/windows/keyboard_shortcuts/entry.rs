use crate::app::actions::Dismiss;
use crate::app::key_caps::KeyCaps;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::primitives::key_cap;
use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};
use jayjay_core::shortcuts::{EffectiveBindings, ShortcutCommand};

pub(super) enum ShortcutEntry {
    Command(ShortcutCommand),
    Dismiss,
}

impl ShortcutEntry {
    fn title(&self) -> String {
        match self {
            Self::Command(command) => command.title(),
            Self::Dismiss => jayjay_i18n::shortcuts::cancel_drag(),
        }
    }

    pub(super) fn is_visible(&self, bindings: &EffectiveBindings) -> bool {
        match self {
            Self::Command(command) => bindings.entry(*command).chords().next().is_some(),
            Self::Dismiss => true,
        }
    }

    fn key_caps(&self, bindings: &EffectiveBindings, cx: &App) -> Vec<String> {
        let Self::Command(command) = self else {
            return KeyCaps::for_action(&Dismiss, cx)
                .map(|caps| caps.caps().to_vec())
                .unwrap_or_default();
        };
        let mut caps = Vec::new();
        for chord in bindings.entry(*command).chords() {
            if !caps.is_empty() {
                caps.push("/".to_owned());
            }
            caps.extend(chord.symbols(bindings.platform()));
        }
        caps
    }

    pub(super) fn render(
        &self,
        id: &str,
        bindings: &EffectiveBindings,
        t: &Theme,
        cx: &App,
    ) -> AnyElement {
        let label = self.title();
        let keys = self.key_caps(bindings, cx);
        let accessible_label = format!("{}: {}", label, keys.join(" "));
        let mut caps = div().flex().items_center().gap(px(4.));
        for (ix, key) in keys.into_iter().enumerate() {
            let selector = format!("shortcut-key-{id}-{ix}-{key}");
            caps = caps.child(
                key_cap(key, t)
                    .id(SharedString::from(selector.clone()))
                    .debug_selector(move || selector),
            );
        }
        let selector = format!("shortcut-entry-{id}");
        div()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector)
            .role(gpui::Role::Label)
            .aria_label(accessible_label)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .text_size(ui_font_size(13.))
            .child(div().min_w_0().child(label))
            .child(caps)
            .into_any_element()
    }
}
