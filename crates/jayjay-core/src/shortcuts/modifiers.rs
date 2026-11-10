use super::platform::KeyPlatform;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub cmd: bool,
}

impl Modifiers {
    pub fn symbols(self, platform: KeyPlatform) -> Vec<String> {
        let names = match platform {
            KeyPlatform::Mac => ["⌃", "⌥", "⇧", "⌘"],
            KeyPlatform::Windows | KeyPlatform::Other => ["Ctrl", "Alt", "Shift", "Super"],
        };
        [self.ctrl, self.alt, self.shift, self.cmd]
            .into_iter()
            .zip(names)
            .filter(|(held, _)| *held)
            .map(|(_, name)| name.to_owned())
            .collect()
    }

    pub(super) fn with(mut self, flag: ModifierFlag) -> Self {
        match flag {
            ModifierFlag::Ctrl => self.ctrl = true,
            ModifierFlag::Alt => self.alt = true,
            ModifierFlag::Shift => self.shift = true,
            ModifierFlag::Cmd => self.cmd = true,
        }
        self
    }
}

#[derive(Clone, Copy)]
pub(super) enum ModifierFlag {
    Ctrl,
    Alt,
    Shift,
    Cmd,
}
