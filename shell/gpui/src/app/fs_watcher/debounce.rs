use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::fs_event::FsEvent;
use super::path_classifier::{EventClass, PathClassifier};

pub(super) const OP_DEBOUNCE: Duration = Duration::from_millis(1000);
pub(super) const WC_DEBOUNCE: Duration = Duration::from_millis(2000);

/// Events inside a window are held, not dropped, and `flush` replays them once the window closes.
pub(super) struct Debounce {
    last_op: Instant,
    last_wc: Instant,
    held_op: bool,
    held_wc_paths: BTreeSet<PathBuf>,
}

impl Debounce {
    pub(super) fn fresh() -> Self {
        Self {
            last_op: Instant::now() - OP_DEBOUNCE,
            last_wc: Instant::now() - WC_DEBOUNCE,
            held_op: false,
            held_wc_paths: BTreeSet::new(),
        }
    }

    fn op_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_op) >= OP_DEBOUNCE
    }

    fn wc_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_wc) >= WC_DEBOUNCE
    }

    /// When the earliest held event becomes due.
    pub(super) fn next_flush(&self) -> Option<Instant> {
        let op = self.held_op.then(|| self.last_op + OP_DEBOUNCE);
        let wc = (!self.held_wc_paths.is_empty()).then(|| self.last_wc + WC_DEBOUNCE);
        op.into_iter().chain(wc).min()
    }

    /// Replays held events whose window has closed; held working-copy paths still pass the relevance filter.
    pub(super) fn flush(
        &mut self,
        now: Instant,
        is_relevant_wc_change: &dyn Fn(&[PathBuf]) -> bool,
    ) -> Vec<FsEvent> {
        let mut events = Vec::new();
        if self.held_op && self.op_ready(now) {
            self.held_op = false;
            self.last_op = now;
            events.push(FsEvent::OpHeads);
        }
        if !self.held_wc_paths.is_empty() && self.wc_ready(now) {
            let paths: Vec<PathBuf> = std::mem::take(&mut self.held_wc_paths)
                .into_iter()
                .collect();
            if is_relevant_wc_change(&paths) {
                self.last_wc = now;
                events.push(FsEvent::WorkingCopy);
            }
        }
        events
    }
}

/// The window is checked before the relevance filter so a build storm never runs the gitignore matcher.
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
            if !guard.op_ready(now) {
                guard.held_op = true;
                return None;
            }
            guard.last_op = now;
            Some(FsEvent::OpHeads)
        }
        EventClass::WorkingCopy => {
            {
                let mut guard = debounce.lock().expect("wc debounce lock");
                if !guard.wc_ready(now) {
                    guard.held_wc_paths.extend(event.paths.iter().cloned());
                    return None;
                }
            }
            if !is_relevant_wc_change(&event.paths) {
                return None;
            }
            let mut guard = debounce.lock().expect("wc debounce lock");
            // Re-check under the lock: a concurrent event may have stamped it meanwhile.
            if !guard.wc_ready(now) {
                guard.held_wc_paths.extend(event.paths.iter().cloned());
                return None;
            }
            guard.last_wc = now;
            Some(FsEvent::WorkingCopy)
        }
    }
}
