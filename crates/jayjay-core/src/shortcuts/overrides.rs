use std::collections::BTreeMap;

use super::assign_error::AssignError;
use super::chord::Chord;
use super::command::ShortcutCommand;
use super::platform::KeyPlatform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Override {
    Unbound,
    Chord(Chord),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShortcutOverrides {
    entries: BTreeMap<String, Override>,
    unknown: BTreeMap<String, Vec<String>>,
}

impl ShortcutOverrides {
    pub fn from_stored(stored: BTreeMap<String, Vec<String>>, platform: KeyPlatform) -> Self {
        let mut overrides = Self::default();
        for (id, values) in stored {
            let Some(command) = ShortcutCommand::from_id(&id) else {
                overrides.unknown.insert(id, values);
                continue;
            };
            if command.is_locked() {
                continue;
            }
            if values.is_empty() {
                overrides.entries.insert(id, Override::Unbound);
                continue;
            }
            for value in &values {
                // A pinned default may be a chord reserved for every other command.
                if let Ok(chord) = Chord::parse(value)
                    && (chord.validate(platform).is_ok()
                        || command.default_chord(platform) == Some(chord))
                {
                    overrides.entries.insert(id, Override::Chord(chord));
                    break;
                }
            }
        }
        overrides
    }

    pub fn to_stored(&self) -> BTreeMap<String, Vec<String>> {
        let mut stored = self.unknown.clone();
        for (id, entry) in &self.entries {
            match entry {
                Override::Unbound => {
                    stored.insert(id.clone(), Vec::new());
                }
                Override::Chord(chord) => {
                    stored.insert(id.clone(), vec![chord.to_string()]);
                }
            }
        }
        stored
    }

    pub fn unbind(&mut self, command: ShortcutCommand) -> Result<(), AssignError> {
        if command.is_locked() {
            return Err(AssignError::LockedCommand);
        }
        self.entries
            .insert(command.id().to_owned(), Override::Unbound);
        Ok(())
    }

    pub fn reset_all(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn entry(&self, command: ShortcutCommand) -> Option<&Override> {
        if command.is_locked() {
            return None;
        }
        self.entries.get(command.id())
    }

    pub(crate) fn remove_entry(&mut self, command: ShortcutCommand) {
        self.entries.remove(command.id());
    }

    /// Stores `chord` even when it is the default, so the command keeps it without the aliases that come with the default.
    pub(crate) fn pin_chord(&mut self, command: ShortcutCommand, chord: Chord) {
        self.entries
            .insert(command.id().to_owned(), Override::Chord(chord));
    }

    pub(crate) fn set_entry(
        &mut self,
        command: ShortcutCommand,
        entry: Override,
        platform: KeyPlatform,
    ) {
        match entry {
            Override::Unbound => {
                self.entries
                    .insert(command.id().to_owned(), Override::Unbound);
            }
            Override::Chord(chord) => {
                if command.default_chord(platform) == Some(chord) {
                    self.entries.remove(command.id());
                } else {
                    self.entries
                        .insert(command.id().to_owned(), Override::Chord(chord));
                }
            }
        }
    }
}
