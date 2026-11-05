use super::pending_edit::PendingEdit;
use super::view::ShortcutSettings;
use crate::app::config;
use crate::app::shortcut_chord::ShortcutChord;
use gpui::{Context, Modifiers, Window};
use jayjay_core::shortcuts::{
    AssignError, ChordError, EffectiveBindings, KeyPlatform, ShortcutCommand, ShortcutOverrides,
};
use jayjay_i18n::shortcuts;

impl ShortcutSettings {
    pub(super) fn record(
        &mut self,
        command: ShortcutCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cancel(cx);
        self.recording = Some(command);
        self.recorder_focus.focus(window, cx);
    }

    pub(crate) fn cancel(&mut self, cx: &mut Context<Self>) {
        self.recording = None;
        self.pending = None;
        self.error = None;
        self.replace_allowed = false;
        self.held = Modifiers::default();
        cx.notify();
    }

    pub(super) fn save(overrides: &ShortcutOverrides, cx: &mut Context<Self>) {
        config::update(cx, |cfg| cfg.shortcuts = overrides.to_stored());
    }

    pub(super) fn capture(&mut self, key: &gpui::Keystroke, cx: &mut Context<Self>) {
        if matches!(
            key.key.as_str(),
            "shift" | "control" | "alt" | "platform" | "function" | "cmd"
        ) {
            return;
        }
        if key.key == "escape" {
            self.cancel(cx);
            return;
        }
        let Some(command) = self.recording else {
            return;
        };
        if matches!(key.key.as_str(), "backspace" | "delete") && !key.modifiers.modified() {
            let mut overrides = config::current(cx).shortcut_overrides();
            if overrides.unbind(command).is_ok() {
                Self::save(&overrides, cx);
                self.cancel(cx);
            }
            return;
        }
        match key.chord() {
            Ok(chord) => self.apply(PendingEdit::Assign(command, chord), false, cx),
            Err(error) => {
                self.error = Some(Self::chord_error(error));
                cx.notify();
            }
        }
    }

    fn chord_error(error: ChordError) -> String {
        match error {
            ChordError::Malformed => shortcuts::invalid_syntax(),
            ChordError::NeedsModifier => shortcuts::needs_modifier(),
            ChordError::Reserved => shortcuts::reserved(),
            ChordError::UnsupportedModifier => shortcuts::unsupported_modifier(),
        }
    }

    pub(super) fn apply(&mut self, edit: PendingEdit, replace: bool, cx: &mut Context<Self>) {
        let mut overrides = config::current(cx).shortcut_overrides();
        let bindings = EffectiveBindings::resolve(&overrides, KeyPlatform::current());
        match edit.apply(&mut overrides, &bindings, replace) {
            Ok(()) => {
                Self::save(&overrides, cx);
                self.cancel(cx);
            }
            Err(AssignError::Invalid(error)) => {
                self.error = Some(Self::chord_error(error));
            }
            Err(error) => {
                self.recording = None;
                self.pending = Some(edit);
                self.replace_allowed = error == AssignError::Conflict;
                let names = edit
                    .chord(&bindings)
                    .map(|chord| {
                        bindings
                            .conflicts(edit.command(), chord)
                            .into_iter()
                            .map(|command| {
                                format!("{} ({})", command.title(), command.context().title())
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                self.error = Some(if self.replace_allowed {
                    shortcuts::conflict(&names)
                } else {
                    shortcuts::locked_conflict(&names)
                });
            }
        }
        cx.notify();
    }
}
