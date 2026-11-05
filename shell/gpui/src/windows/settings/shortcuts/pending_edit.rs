use jayjay_core::shortcuts::{
    AssignError, Chord, EffectiveBindings, ShortcutCommand, ShortcutOverrides,
};

#[derive(Clone, Copy)]
pub(super) enum PendingEdit {
    Assign(ShortcutCommand, Chord),
    Reset(ShortcutCommand),
}

impl PendingEdit {
    pub(super) fn command(self) -> ShortcutCommand {
        match self {
            Self::Assign(command, _) | Self::Reset(command) => command,
        }
    }

    pub(super) fn chord(self, bindings: &EffectiveBindings) -> Option<Chord> {
        match self {
            Self::Assign(_, chord) => Some(chord),
            Self::Reset(command) => command.default_chord(bindings.platform()),
        }
    }

    pub(super) fn apply(
        self,
        overrides: &mut ShortcutOverrides,
        bindings: &EffectiveBindings,
        replace: bool,
    ) -> Result<(), AssignError> {
        match self {
            Self::Assign(command, chord) => overrides.assign(command, chord, bindings, replace),
            Self::Reset(command) => overrides.reset(command, bindings, replace),
        }
    }
}
