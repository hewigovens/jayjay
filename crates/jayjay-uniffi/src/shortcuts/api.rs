use super::binding::ShortcutBinding;
use super::edit::ShortcutEdit;
use super::error::ShortcutEditError;
use jayjay_core::shortcuts::{
    Chord, EffectiveBindings, KeyPlatform, Modifiers, ShortcutCommand, ShortcutOverrides,
};
use std::collections::HashMap;

#[uniffi::export]
fn shortcut_bindings(stored: HashMap<String, Vec<String>>) -> Vec<ShortcutBinding> {
    let overrides = ShortcutOverrides::from_stored(stored.into_iter().collect(), KeyPlatform::Mac);
    let stored = overrides.to_stored();
    EffectiveBindings::resolve(&overrides, KeyPlatform::Mac)
        .entries()
        .iter()
        .map(|entry| ShortcutBinding::new(entry, stored.contains_key(entry.command.id())))
        .collect()
}

#[uniffi::export]
fn edit_shortcut(
    stored: HashMap<String, Vec<String>>,
    id: String,
    edit: ShortcutEdit,
    replace: bool,
) -> Result<HashMap<String, Vec<String>>, ShortcutEditError> {
    let mut overrides =
        ShortcutOverrides::from_stored(stored.into_iter().collect(), KeyPlatform::Mac);
    let bindings = EffectiveBindings::resolve(&overrides, KeyPlatform::Mac);
    let command = || ShortcutCommand::from_id(&id).ok_or(ShortcutEditError::UnknownCommand);
    match edit {
        ShortcutEdit::Assign { chord } => {
            let command = command()?;
            let chord = Chord::parse(&chord)?;
            overrides
                .assign(command, chord, &bindings, replace)
                .map_err(|error| {
                    ShortcutEditError::from_assign(error, command, Some(chord), &bindings)
                })?;
        }
        ShortcutEdit::Unbind => {
            let command = command()?;
            overrides
                .unbind(command)
                .map_err(|error| ShortcutEditError::from_assign(error, command, None, &bindings))?;
        }
        ShortcutEdit::Reset => {
            let command = command()?;
            overrides
                .reset(command, &bindings, replace)
                .map_err(|error| {
                    ShortcutEditError::from_assign(
                        error,
                        command,
                        command.default_chord(KeyPlatform::Mac),
                        &bindings,
                    )
                })?;
        }
        ShortcutEdit::ResetAll => overrides.reset_all(),
    }
    Ok(overrides.to_stored().into_iter().collect())
}

#[uniffi::export]
fn shortcut_modifier_caps(ctrl: bool, alt: bool, shift: bool, cmd: bool) -> Vec<String> {
    Modifiers {
        ctrl,
        alt,
        shift,
        cmd,
    }
    .symbols(KeyPlatform::Mac)
}
