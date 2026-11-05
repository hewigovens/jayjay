use std::collections::BTreeMap;

use crate::shortcuts::{
    AssignError, Chord, EffectiveBindings, KeyPlatform, Override, ShortcutCommand,
    ShortcutOverrides,
};

#[test]
fn assign_replace_unbinds_conflicts_without_restoring_defaults() {
    let platform = KeyPlatform::Mac;
    let revset = Chord::parse_default("mod-l", platform).unwrap();
    let mut overrides = ShortcutOverrides::default();
    let bindings = EffectiveBindings::resolve(&overrides, platform);
    overrides
        .assign(ShortcutCommand::ChangesAbandon, revset, &bindings, true)
        .unwrap();
    let stored = overrides.to_stored();
    let round_trip = ShortcutOverrides::from_stored(stored, platform);
    let resolved = EffectiveBindings::resolve(&round_trip, platform);
    assert_eq!(
        resolved.effective_chord(ShortcutCommand::ChangesAbandon),
        Some(revset)
    );
    assert_eq!(
        resolved.effective_chord(ShortcutCommand::RepoFilterByRevset),
        None
    );
    let bindings = EffectiveBindings::resolve(&overrides, platform);
    overrides
        .assign(ShortcutCommand::RepoRefresh, revset, &bindings, true)
        .unwrap();
    let stored = overrides.to_stored();
    assert!(!stored.contains_key(ShortcutCommand::ChangesAbandon.id()));
    assert_eq!(
        stored.get(ShortcutCommand::RepoFilterByRevset.id()),
        Some(&Vec::new())
    );
}

#[test]
fn reset_restored_default_checks_conflicts() {
    let platform = KeyPlatform::Mac;
    let revset = Chord::parse_default("mod-l", platform).unwrap();
    let mut overrides = ShortcutOverrides::default();
    let bindings = EffectiveBindings::resolve(&overrides, platform);
    overrides
        .assign(ShortcutCommand::ChangesAbandon, revset, &bindings, true)
        .unwrap();
    let bindings = EffectiveBindings::resolve(&overrides, platform);
    assert_eq!(
        bindings.effective_chord(ShortcutCommand::RepoFilterByRevset),
        None
    );
    assert!(matches!(
        overrides.reset(ShortcutCommand::RepoFilterByRevset, &bindings, false),
        Err(AssignError::Conflict)
    ));
    overrides
        .reset(ShortcutCommand::RepoFilterByRevset, &bindings, true)
        .unwrap();
    let bindings = EffectiveBindings::resolve(&overrides, platform);
    assert_eq!(
        bindings.effective_chord(ShortcutCommand::RepoFilterByRevset),
        Some(revset)
    );
    assert_eq!(
        bindings.effective_chord(ShortcutCommand::ChangesAbandon),
        None
    );
}

#[test]
fn stored_round_trip_semantics() {
    let platform = KeyPlatform::Mac;
    let custom = Chord::parse("shift-cmd-k").unwrap();
    let stored = BTreeMap::from([
        ("legacy.unknown".to_owned(), vec!["ctrl-x".to_owned()]),
        (ShortcutCommand::RepoRefresh.id().to_owned(), Vec::new()),
        (
            ShortcutCommand::AppSettings.id().to_owned(),
            vec!["mod-x".to_owned()],
        ),
        (
            ShortcutCommand::RepoBookmarkManager.id().to_owned(),
            vec!["not-valid".to_owned(), custom.to_string()],
        ),
        (
            ShortcutCommand::RepoFetch.id().to_owned(),
            vec!["not-valid".to_owned()],
        ),
    ]);
    let overrides = ShortcutOverrides::from_stored(stored, platform);
    assert_eq!(
        overrides.entry(ShortcutCommand::RepoRefresh),
        Some(&Override::Unbound)
    );
    assert_eq!(
        EffectiveBindings::resolve(&overrides, platform)
            .effective_chord(ShortcutCommand::RepoBookmarkManager),
        Some(custom)
    );
    assert_eq!(
        EffectiveBindings::resolve(&overrides, platform)
            .effective_chord(ShortcutCommand::AppSettings),
        ShortcutCommand::AppSettings.default_chord(platform)
    );
    let out = overrides.to_stored();
    assert!(!out.contains_key(ShortcutCommand::AppSettings.id()));
    assert!(!out.contains_key(ShortcutCommand::RepoFetch.id()));
    let mut overrides = overrides;
    overrides.reset_all();
    let out = overrides.to_stored();
    assert_eq!(
        out.get("legacy.unknown").map(|values| values.as_slice()),
        Some(["ctrl-x".to_owned()].as_slice())
    );
    assert!(!out.contains_key(ShortcutCommand::RepoRefresh.id()));
}
