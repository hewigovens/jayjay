use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::fs_event::FsEvent;
use super::path_classifier::{EventClass, PathClassifier};

pub(super) const OP_DEBOUNCE: Duration = Duration::from_millis(1000);
pub(super) const WC_DEBOUNCE: Duration = Duration::from_millis(2000);

/// Leading-edge debounce timestamps for the two event streams.
pub(super) struct Debounce {
    last_op: Instant,
    last_wc: Instant,
}

impl Debounce {
    pub(super) fn fresh() -> Self {
        Self {
            last_op: Instant::now() - OP_DEBOUNCE,
            last_wc: Instant::now() - WC_DEBOUNCE,
        }
    }

    /// Whether enough time has elapsed to emit another op-heads refresh.
    fn op_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_op) >= OP_DEBOUNCE
    }

    /// Whether enough time has elapsed to emit another working-copy refresh.
    fn wc_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_wc) >= WC_DEBOUNCE
    }
}

/// Decide whether a raw event should emit, stamping the debounce on a send. The window is
/// checked before the relevance filter so a build storm is dropped without running the
/// gitignore matcher more than once per `WC_DEBOUNCE`.
pub(super) fn next_event(
    classifier: &PathClassifier,
    debounce: &Mutex<Debounce>,
    event: &notify::Event,
    now: Instant,
    is_relevant_wc_change: &dyn Fn(&[PathBuf]) -> bool,
) -> Option<FsEvent> {
    match classifier.classify(event) {
        EventClass::Ignore => None,
        EventClass::OpHeads => {
            let mut guard = debounce.lock().expect("op debounce lock");
            guard.op_ready(now).then(|| {
                guard.last_op = now;
                FsEvent::OpHeads
            })
        }
        EventClass::WorkingCopy => {
            // Read-only window check first; bail before touching the gitignore matcher.
            {
                let guard = debounce.lock().expect("wc debounce lock");
                if !guard.wc_ready(now) {
                    return None;
                }
            }
            if !is_relevant_wc_change(&event.paths) {
                return None;
            }
            let mut guard = debounce.lock().expect("wc debounce lock");
            // Re-check under the lock: a concurrent event may have stamped it meanwhile.
            guard.wc_ready(now).then(|| {
                guard.last_wc = now;
                FsEvent::WorkingCopy
            })
        }
    }
}
