#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordError {
    Malformed,
    NeedsModifier,
    Reserved,
    UnsupportedModifier,
}
