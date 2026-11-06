use jayjay_core::shortcuts::ChordError;

#[derive(Debug, uniffi::Enum)]
pub enum ShortcutChordError {
    Malformed,
    NeedsModifier,
    Reserved,
    UnsupportedModifier,
}

impl From<ChordError> for ShortcutChordError {
    fn from(error: ChordError) -> Self {
        match error {
            ChordError::Malformed => Self::Malformed,
            ChordError::NeedsModifier => Self::NeedsModifier,
            ChordError::Reserved => Self::Reserved,
            ChordError::UnsupportedModifier => Self::UnsupportedModifier,
        }
    }
}
