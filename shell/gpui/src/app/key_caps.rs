use super::shortcut_chord::ShortcutChord;
use gpui::{Action, App, Modifiers};
use jayjay_core::shortcuts::{Chord, KeyPlatform};

#[derive(Debug)]
pub struct KeyCaps(Vec<String>);

impl KeyCaps {
    pub fn for_action(action: &dyn Action, cx: &App) -> Option<Self> {
        let keymap = cx.key_bindings();
        let keymap = keymap.borrow();
        let binding = keymap.bindings_for_action(action).last()?;
        let [keystroke] = binding.keystrokes() else {
            return None;
        };
        keystroke.inner().chord().ok().map(Self::from)
    }

    pub fn label(action: &dyn Action, cx: &App) -> String {
        Self::for_action(action, cx)
            .map(|caps| caps.to_string())
            .unwrap_or_default()
    }

    pub(crate) fn tooltip(title: String, action: &dyn Action, cx: &App) -> String {
        match Self::for_action(action, cx) {
            Some(caps) => format!("{title} ({caps})"),
            None => title,
        }
    }

    pub fn caps(&self) -> &[String] {
        &self.0
    }
}

impl From<&Modifiers> for KeyCaps {
    fn from(modifiers: &Modifiers) -> Self {
        let modifiers = jayjay_core::shortcuts::Modifiers {
            ctrl: modifiers.control,
            alt: modifiers.alt,
            shift: modifiers.shift,
            cmd: modifiers.platform,
        };
        Self(modifiers.symbols(KeyPlatform::current()))
    }
}

impl From<Chord> for KeyCaps {
    fn from(chord: Chord) -> Self {
        Self(chord.symbols(KeyPlatform::current()))
    }
}

impl std::fmt::Display for KeyCaps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let separator = if cfg!(target_os = "macos") { "" } else { "+" };
        f.write_str(&self.0.join(separator))
    }
}
