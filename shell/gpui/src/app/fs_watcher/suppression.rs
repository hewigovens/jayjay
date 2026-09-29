use gpui::{App, Global};

/// When set, the real `notify` OS thread isn't spawned, so the FSEvents loop can't trip the GPUI scheduler's nondeterminism guard in tests.
#[derive(Default)]
struct WatcherSuppressed(bool);

impl Global for WatcherSuppressed {}

pub(crate) fn is_watcher_suppressed(cx: &App) -> bool {
    cx.try_global::<WatcherSuppressed>().is_some_and(|s| s.0)
}

pub fn suppress_for_tests(cx: &mut App) {
    cx.set_global(WatcherSuppressed(true));
}
