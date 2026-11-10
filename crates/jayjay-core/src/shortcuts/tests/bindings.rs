use crate::shortcuts::{
    AssignError, Chord, ChordError, EffectiveBindings, KeyPlatform, ShortcutCommand,
    ShortcutOverrides,
};

#[test]
fn conflict_detection() {
    let platform = KeyPlatform::Mac;
    let bindings = EffectiveBindings::resolve(&ShortcutOverrides::default(), platform);
    let find = Chord::parse_default("mod-f", platform).unwrap();
    let revset = Chord::parse_default("mod-l", platform).unwrap();
    assert!(
        bindings
            .conflicts(ShortcutCommand::RepoFindInDiff, find)
            .is_empty()
    );
    assert!(
        bindings
            .conflicts(ShortcutCommand::ChangesAbandon, revset)
            .contains(&ShortcutCommand::RepoFilterByRevset)
    );
    let locked = Chord::parse("j").unwrap();
    let mut overrides = ShortcutOverrides::default();
    let bindings = EffectiveBindings::resolve(&overrides, platform);
    assert!(matches!(
        overrides.assign(ShortcutCommand::ChangesAbandon, locked, &bindings, true,),
        Err(AssignError::Invalid(ChordError::NeedsModifier))
    ));
    let navigation = Chord::parse("ctrl-n").unwrap();
    assert!(matches!(
        overrides.assign(
            ShortcutCommand::RepoRefresh,
            navigation,
            &EffectiveBindings::resolve(&overrides, platform),
            true,
        ),
        Err(AssignError::LockedConflict)
    ));
    assert!(matches!(
        overrides.unbind(ShortcutCommand::AppQuit),
        Err(AssignError::LockedCommand)
    ));
}

#[test]
fn redo_takes_ctrl_y_only_on_windows_and_drops_it_once_customized() {
    let ctrl_y = Chord::parse("ctrl-y").unwrap();
    for (platform, aliased) in [
        (KeyPlatform::Windows, true),
        (KeyPlatform::Other, false),
        (KeyPlatform::Mac, false),
    ] {
        let bindings = EffectiveBindings::resolve(&ShortcutOverrides::default(), platform);
        assert_eq!(
            bindings
                .entry(ShortcutCommand::RepoRedo)
                .aliases
                .contains(&ctrl_y),
            aliased,
            "{platform:?}"
        );
    }
    let platform = KeyPlatform::Windows;
    let mut overrides = ShortcutOverrides::default();
    let custom = Chord::parse("ctrl-alt-y").unwrap();
    overrides
        .assign(
            ShortcutCommand::RepoRedo,
            custom,
            &EffectiveBindings::resolve(&overrides, platform),
            false,
        )
        .unwrap();
    let redo = EffectiveBindings::resolve(&overrides, platform)
        .entry(ShortcutCommand::RepoRedo)
        .clone();
    assert_eq!(redo.chords().collect::<Vec<_>>(), vec![custom]);
}
