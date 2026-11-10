use super::assign_error::AssignError;
use super::bindings::EffectiveBindings;
use super::chord::Chord;
use super::command::ShortcutCommand;
use super::overrides::{Override, ShortcutOverrides};

impl ShortcutOverrides {
    pub fn assign(
        &mut self,
        command: ShortcutCommand,
        chord: Chord,
        bindings: &EffectiveBindings,
        replace: bool,
    ) -> Result<(), AssignError> {
        if command.is_locked() {
            return Err(AssignError::LockedCommand);
        }
        let platform = bindings.platform();
        // The default chord brings its aliases back, and it may be reserved for every other command.
        if command.default_chord(platform) == Some(chord) {
            return self.reset(command, bindings, replace);
        }
        chord.validate(platform).map_err(AssignError::Invalid)?;
        clear_conflicts(self, command, chord, bindings, replace)?;
        self.set_entry(command, Override::Chord(chord), platform);
        Ok(())
    }

    pub fn reset(
        &mut self,
        command: ShortcutCommand,
        bindings: &EffectiveBindings,
        replace: bool,
    ) -> Result<(), AssignError> {
        if command.is_locked() {
            return Err(AssignError::LockedCommand);
        }
        let platform = bindings.platform();
        for chord in command
            .default_chord(platform)
            .into_iter()
            .chain(command.aliases(platform))
        {
            clear_conflicts(self, command, chord, bindings, replace)?;
        }
        self.remove_entry(command);
        Ok(())
    }
}

fn clear_conflicts(
    overrides: &mut ShortcutOverrides,
    command: ShortcutCommand,
    chord: Chord,
    bindings: &EffectiveBindings,
    replace: bool,
) -> Result<(), AssignError> {
    let conflicts = bindings.conflicts(command, chord);
    if conflicts.iter().any(|other| other.is_locked()) {
        return Err(AssignError::LockedConflict);
    }
    if !conflicts.is_empty() && !replace {
        return Err(AssignError::Conflict);
    }
    let platform = bindings.platform();
    for other in conflicts {
        match bindings.effective_chord(other) {
            Some(primary) if primary != chord => overrides.pin_chord(other, primary),
            _ if other.default_chord(platform).is_some() => {
                overrides.set_entry(other, Override::Unbound, platform);
            }
            _ => overrides.remove_entry(other),
        }
    }
    Ok(())
}
