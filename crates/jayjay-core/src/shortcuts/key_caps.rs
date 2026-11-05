use super::KeyPlatform;
use super::chord::Chord;
use super::key::Key;

impl Chord {
    pub fn symbols(self, platform: KeyPlatform) -> Vec<String> {
        let mut caps = self.modifiers.symbols(platform);
        caps.push(match platform {
            KeyPlatform::Mac => self.mac_key_cap(),
            KeyPlatform::Other => self.other_key_cap(),
        });
        caps
    }

    fn mac_key_cap(self) -> String {
        match self.key {
            Key::Enter => "↩".to_owned(),
            Key::Backspace => "⌫".to_owned(),
            Key::Delete => "⌦".to_owned(),
            Key::Left => "←".to_owned(),
            Key::Right => "→".to_owned(),
            Key::Up => "↑".to_owned(),
            Key::Down => "↓".to_owned(),
            Key::Char('-') => "−".to_owned(),
            Key::Char(c) if c.is_ascii_alphabetic() => c.to_ascii_uppercase().to_string(),
            Key::Char(c) => c.to_string(),
            Key::F(n) => format!("F{n}"),
            Key::Space => "Space".to_owned(),
            Key::Tab => "Tab".to_owned(),
            Key::Escape => "Esc".to_owned(),
            Key::Home => "Home".to_owned(),
            Key::End => "End".to_owned(),
            Key::PageUp => "PgUp".to_owned(),
            Key::PageDown => "PgDn".to_owned(),
            Key::Menu => "Menu".to_owned(),
        }
    }

    fn other_key_cap(self) -> String {
        match self.key {
            Key::Enter => "Enter".to_owned(),
            Key::Backspace => "Backspace".to_owned(),
            Key::Delete => "Delete".to_owned(),
            Key::Left => "Left".to_owned(),
            Key::Right => "Right".to_owned(),
            Key::Up => "Up".to_owned(),
            Key::Down => "Down".to_owned(),
            Key::Char(c) if c.is_ascii_alphabetic() => c.to_ascii_uppercase().to_string(),
            Key::Char(c) => c.to_string(),
            Key::F(n) => format!("F{n}"),
            Key::Space => "Space".to_owned(),
            Key::Tab => "Tab".to_owned(),
            Key::Escape => "Esc".to_owned(),
            Key::Home => "Home".to_owned(),
            Key::End => "End".to_owned(),
            Key::PageUp => "PageUp".to_owned(),
            Key::PageDown => "PageDown".to_owned(),
            Key::Menu => "Menu".to_owned(),
        }
    }
}
