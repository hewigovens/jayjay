use std::time::Duration;

use gpui::{App, Global};

#[derive(Default)]
struct ReduceMotion(bool);

impl Global for ReduceMotion {}

pub fn reduce_for_tests(cx: &mut App) {
    cx.set_global(ReduceMotion(true));
}

pub(crate) fn animation_duration(cx: &App, duration: Duration) -> Duration {
    if cx
        .try_global::<ReduceMotion>()
        .is_some_and(|reduce| reduce.0)
    {
        Duration::ZERO
    } else {
        duration
    }
}
