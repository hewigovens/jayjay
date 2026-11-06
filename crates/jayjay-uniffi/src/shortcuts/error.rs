use jayjay_core::shortcuts::{AssignError, Chord, ChordError, EffectiveBindings, ShortcutCommand};

use super::chord::ShortcutChord;
use super::chord_error::ShortcutChordError;
use super::conflict::ShortcutConflict;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ShortcutEditError {
    #[error("Invalid shortcut: {kind:?}")]
    Invalid { kind: ShortcutChordError },
    #[error("Shortcut conflict")]
    Conflict {
        chord: Option<ShortcutChord>,
        commands: Vec<ShortcutConflict>,
    },
    #[error("Locked shortcut conflict")]
    LockedConflict {
        chord: Option<ShortcutChord>,
        commands: Vec<ShortcutConflict>,
    },
    #[error("Locked command")]
    LockedCommand,
    #[error("Unknown command")]
    UnknownCommand,
}

impl From<ChordError> for ShortcutEditError {
    fn from(error: ChordError) -> Self {
        Self::Invalid { kind: error.into() }
    }
}

impl ShortcutEditError {
    pub(super) fn from_assign(
        error: AssignError,
        command: ShortcutCommand,
        chord: Option<Chord>,
        bindings: &EffectiveBindings,
    ) -> Self {
        let commands = chord
            .map(|chord| bindings.conflicts(command, chord))
            .unwrap_or_default()
            .into_iter()
            .map(|command| ShortcutConflict {
                title: command.title(),
                context_title: command.context().title(),
            })
            .collect();
        match error {
            AssignError::Invalid(error) => error.into(),
            AssignError::Conflict => Self::Conflict {
                chord: chord.map(Into::into),
                commands,
            },
            AssignError::LockedConflict => Self::LockedConflict {
                chord: chord.map(Into::into),
                commands,
            },
            AssignError::LockedCommand => Self::LockedCommand,
        }
    }
}
