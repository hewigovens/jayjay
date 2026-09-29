use gpui::{App, Global};

/// When set, the watcher is armed but the real `notify` OS thread isn't spawned — tests
/// install this so the FSEvents loop can't trip the GPUI scheduler's nondeterminism guard.
#[derive(Default)]
struct WatcherSuppressed(bool);

impl Global for WatcherSuppressed {}

/// True when the real OS watcher must not be spawned (test scheduler is active).
pub(crate) fn is_watcher_suppressed(cx: &App) -> bool {
    cx.try_global::<WatcherSuppressed>().is_some_and(|s| s.0)
}

/// Suppress real OS-thread watchers for the rest of this process. Tests call this.
pub fn suppress_for_tests(cx: &mut App) {
    cx.set_global(WatcherSuppressed(true));
}
