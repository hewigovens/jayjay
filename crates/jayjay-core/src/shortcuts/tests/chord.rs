use crate::shortcuts::{Chord, ChordError, KeyPlatform};

#[test]
fn chord_parse_display_round_trip_and_mod_expansion() {
    for platform in [KeyPlatform::Mac, KeyPlatform::Other] {
        let chord = Chord::parse_default("shift-mod-p", platform).unwrap();
        let text = chord.to_string();
        assert_eq!(Chord::parse(&text).unwrap(), chord);
        let expanded_mod = Chord::parse_default("mod-o", platform).unwrap();
        assert!(expanded_mod.modifiers.cmd == (platform == KeyPlatform::Mac));
        assert!(expanded_mod.modifiers.ctrl == (platform == KeyPlatform::Other));
    }
    assert!(matches!(
        Chord::parse("mod-o"),
        Err(ChordError::UnsupportedModifier)
    ));
    assert!(matches!(
        Chord::parse("not-a-real-key"),
        Err(ChordError::Malformed)
    ));
    let reordered = Chord::parse("alt-ctrl-s").unwrap();
    assert_eq!(reordered.to_string(), "ctrl-alt-s");
    for shifted in ["cmd-?", "cmd-!", "cmd-ö"] {
        assert_eq!(Chord::parse(shifted).unwrap().to_string(), shifted);
    }
}

#[test]
fn chord_validate_rules() {
    assert!(matches!(
        Chord::parse("j").unwrap().validate(KeyPlatform::Mac),
        Err(ChordError::NeedsModifier)
    ));
    assert!(
        Chord::parse("f6")
            .unwrap()
            .validate(KeyPlatform::Mac)
            .is_ok()
    );
    assert!(matches!(
        Chord::parse("f5").unwrap().validate(KeyPlatform::Mac),
        Err(ChordError::Reserved)
    ));
    assert!(matches!(
        Chord::parse("shift-a").unwrap().validate(KeyPlatform::Mac),
        Err(ChordError::NeedsModifier)
    ));
    assert!(matches!(
        Chord::parse("cmd-k").unwrap().validate(KeyPlatform::Other),
        Err(ChordError::UnsupportedModifier)
    ));
    assert!(matches!(
        Chord::parse_default("mod-q", KeyPlatform::Mac)
            .unwrap()
            .validate(KeyPlatform::Mac),
        Err(ChordError::Reserved)
    ));
    assert!(matches!(
        Chord::parse_default("alt-f4", KeyPlatform::Other)
            .unwrap()
            .validate(KeyPlatform::Other),
        Err(ChordError::Reserved)
    ));
    assert!(
        Chord::parse("ctrl-h")
            .unwrap()
            .validate(KeyPlatform::Other)
            .is_ok()
    );
    assert!(
        Chord::parse("alt-f4")
            .unwrap()
            .validate(KeyPlatform::Mac)
            .is_ok()
    );
}
