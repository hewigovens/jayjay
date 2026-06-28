//! Deterministic `IdSource`/`Clock` impls. They must live in this crate — a helper crate linking jayjay-review would implement different trait types than the in-test `crate::` ones — and the `test-util` feature lets other crates' tests reuse them.

use crate::{Clock, IdSource};

pub struct SequentialIds {
    next: u32,
}

impl SequentialIds {
    pub fn new() -> Self {
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

pub struct FixedClock(pub i64);

impl Clock for FixedClock {
    fn now_ms(&self) -> i64 {
        self.0
    }
}
