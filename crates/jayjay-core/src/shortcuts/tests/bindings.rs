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
