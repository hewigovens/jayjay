use std::fmt;

use super::KeyPlatform;
use super::chord_error::ChordError;
use super::key::Key;
use super::modifiers::{ModifierFlag, Modifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Chord {
    pub fn parse(text: &str) -> Result<Self, ChordError> {
        Self::parse_tokens(text, false, KeyPlatform::Mac)
    }

    pub fn parse_default(text: &str, platform: KeyPlatform) -> Result<Self, ChordError> {
        Self::parse_tokens(text, true, platform)
    }

    fn parse_tokens(
        text: &str,
        allow_mod: bool,
        platform: KeyPlatform,
    ) -> Result<Self, ChordError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ChordError::Malformed);
        }
        let mut parts = text.split('-').peekable();
        let mut modifiers = Modifiers::default();
        while let Some(part) = parts.peek() {
            match Self::modifier_token(part, allow_mod, platform)? {
                Some(flag) => {
                    modifiers = modifiers.with(flag);
                    parts.next();
                }
                None => break,
            }
        }
        let key_text = parts.collect::<Vec<_>>().join("-");
        if key_text.is_empty() {
            return Err(ChordError::Malformed);
        }
        let key = Key::from_token(&key_text).ok_or(ChordError::Malformed)?;
        Ok(Self { modifiers, key })
    }

    fn modifier_token(
        token: &str,
        allow_mod: bool,
        platform: KeyPlatform,
    ) -> Result<Option<ModifierFlag>, ChordError> {
        match token {
            "ctrl" | "control" => Ok(Some(ModifierFlag::Ctrl)),
            "alt" | "option" => Ok(Some(ModifierFlag::Alt)),
            "shift" => Ok(Some(ModifierFlag::Shift)),
            "cmd" | "command" | "super" | "platform" => Ok(Some(ModifierFlag::Cmd)),
            "mod" if allow_mod => Ok(Some(match platform {
                KeyPlatform::Mac => ModifierFlag::Cmd,
                KeyPlatform::Other => ModifierFlag::Ctrl,
            })),
            "mod" => Err(ChordError::UnsupportedModifier),
            _ => Ok(None),
        }
    }

    pub fn validate(self, platform: KeyPlatform) -> Result<(), ChordError> {
        if platform == KeyPlatform::Other && self.modifiers.cmd {
            return Err(ChordError::UnsupportedModifier);
        }
        let has_chord_modifier = self.modifiers.ctrl
            || self.modifiers.alt
            || self.modifiers.cmd
            || matches!(self.key, Key::F(_));
        if !has_chord_modifier {
            return Err(ChordError::NeedsModifier);
        }
        if self.is_reserved(platform) {
            return Err(ChordError::Reserved);
        }
        Ok(())
    }

    fn is_reserved(self, platform: KeyPlatform) -> bool {
        let platform_specs = match platform {
            KeyPlatform::Mac => RESERVED_MAC,
            KeyPlatform::Other => RESERVED_OTHER,
        };
        RESERVED_ALL
            .iter()
            .chain(platform_specs)
            .any(|spec| Self::parse_default(spec, platform).ok() == Some(self))
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        if self.modifiers.ctrl {
            write_modifier(f, "ctrl", first)?;
            first = false;
        }
        if self.modifiers.alt {
            write_modifier(f, "alt", first)?;
            first = false;
        }
        if self.modifiers.shift {
            write_modifier(f, "shift", first)?;
            first = false;
        }
        if self.modifiers.cmd {
            write_modifier(f, "cmd", first)?;
            first = false;
        }
        if first {
            self.key.write_token(f)
        } else {
            write!(f, "-")?;
            self.key.write_token(f)
        }
    }
}

fn write_modifier(f: &mut fmt::Formatter<'_>, name: &str, first: bool) -> fmt::Result {
    if !first {
        write!(f, "-")?;
    }
    write!(f, "{name}")
}

const RESERVED_ALL: &[&str] = &[
    "mod-a",
    "mod-c",
    "mod-v",
    "mod-x",
    "mod-z",
    "shift-mod-z",
    "tab",
    "shift-tab",
    "enter",
    "escape",
    "alt-escape",
    "f5",
];

const RESERVED_MAC: &[&str] = &["mod-q", "mod-w", "mod-h", "mod-m", "mod-,", "mod-`"];

const RESERVED_OTHER: &[&str] = &["ctrl-q", "ctrl-w", "ctrl-,", "alt-f4"];
