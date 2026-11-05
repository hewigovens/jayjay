use gpui::Keystroke;
use jayjay_core::shortcuts::{Chord, ChordError};

pub(crate) trait ShortcutChord {
    fn chord(&self) -> Result<Chord, ChordError>;
}

impl ShortcutChord for Keystroke {
    fn chord(&self) -> Result<Chord, ChordError> {
        if self.modifiers.function {
            return Err(ChordError::UnsupportedModifier);
        }
        let mut spec = String::new();
        for (held, name) in [
            (self.modifiers.control, "ctrl-"),
            (self.modifiers.alt, "alt-"),
            (self.modifiers.shift, "shift-"),
            (self.modifiers.platform, "cmd-"),
        ] {
            if held {
                spec.push_str(name);
            }
        }
        spec.push_str(&self.key);
        Chord::parse(&spec)
    }
}
