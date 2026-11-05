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
        let Some(default) = command.default_chord(platform) else {
            self.remove_entry(command);
            return Ok(());
        };
        clear_conflicts(self, command, default, bindings, replace)?;
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
        if other.default_chord(platform).is_some() {
            overrides.set_entry(other, Override::Unbound, platform);
        } else {
            overrides.remove_entry(other);
        }
    }
    Ok(())
}
