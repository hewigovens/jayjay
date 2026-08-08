//! Deterministic `IdSource` impl. It must live in this crate — a helper crate linking jayjay-review would implement a different trait type than the in-test `crate::` one — and the `test-util` feature lets other crates' tests reuse it.

use crate::IdSource;

pub struct SequentialIds {
    next: u32,
}

impl SequentialIds {
    pub(crate) fn new() -> Self {
        Self { next: 1 }
    }
}

impl Default for SequentialIds {
    fn default() -> Self {
        Self::new()
    }
}

impl IdSource for SequentialIds {
    fn next_id(&mut self) -> String {
        let id = format!("note-{}", self.next);
        self.next += 1;
        id
    }
}
