use super::chord_error::ChordError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignError {
    LockedCommand,
    Invalid(ChordError),
    Conflict,
    LockedConflict,
}
