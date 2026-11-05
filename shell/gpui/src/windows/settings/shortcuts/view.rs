use super::pending_edit::PendingEdit;
use crate::app::config::{self, AppConfigStore};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::text_area::{TextArea, TextAreaUpdated};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, Modifiers,
    ModifiersChangedEvent, ParentElement, Render, Styled, Subscription, Window, div, px,
};
use jayjay_core::shortcuts::{
    EffectiveBindings, EffectiveEntry, KeyPlatform, ShortcutCommand, ShortcutContext,
};
use jayjay_i18n::shortcuts;

pub(crate) struct ShortcutSettings {
    pub(super) focus: FocusHandle,
    pub(super) recorder_focus: FocusHandle,
    pub(super) query: Entity<TextArea>,
    pub(super) modified_only: bool,
    pub(super) recording: Option<ShortcutCommand>,
    pub(super) pending: Option<PendingEdit>,
    pub(super) error: Option<String>,
    pub(super) replace_allowed: bool,
    pub(super) held: Modifiers,
    _interceptor: Subscription,
}

impl ShortcutSettings {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| {
            TextArea::new("", shortcuts::search_actions(), false, 28., cx)
                .leading_inset(26.)
                .tab_stop(true)
        });
        cx.subscribe(&query, |_, _, _: &TextAreaUpdated, cx| cx.notify())
            .detach();
        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
            .detach();
        let listener = cx.listener(
            |this: &mut Self, event: &gpui::KeystrokeEvent, window, cx| {
                if this.recording.is_some() && this.recorder_focus.is_focused(window) {
                    cx.stop_propagation();
                    window.prevent_default();
                    this.capture(&event.keystroke, cx);
                }
            },
        );
        let interceptor = cx.intercept_keystrokes(listener);
        Self {
            focus: cx.focus_handle(),
            recorder_focus: cx.focus_handle(),
            query,
            modified_only: false,
            recording: None,
            pending: None,
            error: None,
            replace_allowed: false,
            held: Modifiers::default(),
            _interceptor: interceptor,
        }
    }
}

impl Render for ShortcutSettings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.global::<Theme>().clone();
        let overrides = config::current(cx).shortcut_overrides();
        let stored = overrides.to_stored();
        let bindings = EffectiveBindings::resolve(&overrides, KeyPlatform::current());
        let query = self.query.read(cx).text().to_lowercase();
        let body = div()
            .track_focus(&self.focus)
            .on_modifiers_changed(cx.listener(|this, event: &ModifiersChangedEvent, _, cx| {
                if this.recording.is_some() {
                    this.held = event.modifiers;
                    cx.notify();
                }
            }))
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_size(ui_font_size(12.))
            .child(self.filter_bar(&t, cx));
        let mut groups: Vec<(ShortcutContext, Vec<&EffectiveEntry>)> = Vec::new();
        for entry in bindings
            .entries()
            .iter()
            .filter(|entry| self.shows(entry, &query, stored.contains_key(entry.command.id())))
        {
            let context = entry.command.context();
            match groups.iter_mut().find(|(group, _)| *group == context) {
                Some((_, entries)) => entries.push(entry),
                None => groups.push((context, vec![entry])),
            }
        }
        let mut list = div().flex().flex_col();
        for (context, entries) in groups {
            list = list.child(Self::group_title(context, &t));
            for entry in entries {
                let modified = stored.contains_key(entry.command.id());
                list = list.child(self.row(entry, modified, &bindings, &t, cx));
            }
        }
        body.child(list).when(query.is_empty(), |body| {
            body.child(div().flex().child(Self::reset_all_button(&t, cx)))
        })
    }
}
