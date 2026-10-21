use std::time::{Duration, Instant};

use gpui::App;

const TICK: Duration = Duration::from_millis(100);
const STALL: Duration = Duration::from_millis(250);
const WATCH_FOR: Duration = Duration::from_secs(60);

/// A foreground tick that resumes late means the main thread was blocked.
pub(crate) fn start(launched_at: Instant, cx: &mut App) {
    let mut due = Instant::now();
    cx.spawn(async move |cx| {
        while launched_at.elapsed() < WATCH_FOR {
            let late = Instant::now().saturating_duration_since(due);
            if late >= STALL {
                log::warn!(
                    "main thread stalled {}ms, ending {}ms after launch",
                    late.as_millis(),
                    launched_at.elapsed().as_millis()
                );
            }
            due = Instant::now() + TICK;
            cx.background_executor().timer(TICK).await;
        }
    })
    .detach();
}
