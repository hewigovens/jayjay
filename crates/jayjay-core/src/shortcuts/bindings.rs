use super::chord::Chord;
use super::command::ShortcutCommand;
use super::overrides::{Override, ShortcutOverrides};
use super::platform::KeyPlatform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveEntry {
    pub command: ShortcutCommand,
    pub chord: Option<Chord>,
    pub aliases: Vec<Chord>,
}

impl EffectiveEntry {
    pub fn chords(&self) -> impl Iterator<Item = Chord> + '_ {
        self.chord.iter().chain(&self.aliases).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveBindings {
    platform: KeyPlatform,
    entries: Vec<EffectiveEntry>,
}

impl EffectiveBindings {
    pub fn resolve(overrides: &ShortcutOverrides, platform: KeyPlatform) -> Self {
        let entries = ShortcutCommand::all()
            .iter()
            .map(|&command| match overrides.entry(command) {
                // A customized command keeps only the chord the user chose; its aliases come with the default.
                Some(entry) => EffectiveEntry {
                    command,
                    chord: match entry {
                        Override::Unbound => None,
                        Override::Chord(chord) => Some(*chord),
                    },
                    aliases: Vec::new(),
                },
                None => EffectiveEntry {
                    command,
                    chord: command.default_chord(platform),
                    aliases: command.aliases(platform),
                },
            })
            .collect();
        Self { platform, entries }
    }

    pub fn platform(&self) -> KeyPlatform {
        self.platform
    }

    pub fn entries(&self) -> &[EffectiveEntry] {
        &self.entries
    }

    pub fn entry(&self, command: ShortcutCommand) -> &EffectiveEntry {
        self.entries
            .iter()
            .find(|entry| entry.command == command)
            .expect("bindings resolve every command")
    }

    pub fn effective_chord(&self, command: ShortcutCommand) -> Option<Chord> {
        self.entry(command).chord
    }

    pub fn conflicts(&self, command: ShortcutCommand, chord: Chord) -> Vec<ShortcutCommand> {
        let context = command.context();
        self.entries
            .iter()
            .filter(|entry| entry.command != command)
            .filter(|entry| context.overlaps(entry.command.context()))
            .filter(|entry| entry.chords().any(|bound| bound == chord))
            .map(|entry| entry.command)
            .collect()
    }
}
