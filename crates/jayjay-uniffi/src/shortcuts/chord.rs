use jayjay_core::shortcuts::{Chord, KeyPlatform};

#[derive(Debug, uniffi::Record)]
pub struct ShortcutChord {
    pub text: String,
    pub caps: Vec<String>,
}

impl From<Chord> for ShortcutChord {
    fn from(chord: Chord) -> Self {
        Self {
            text: chord.to_string(),
            caps: chord.symbols(KeyPlatform::Mac),
        }
    }
}
