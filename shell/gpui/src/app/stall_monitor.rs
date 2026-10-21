use std::time::{Duration, Instant};

use gpui::{App, Task};

const TICK: Duration = Duration::from_millis(100);
const STALL: Duration = Duration::from_millis(250);
const SUSPEND: Duration = Duration::from_secs(30);

/// A foreground tick that resumes late means the main thread was blocked.
pub(crate) struct StallMonitor {
    launched_at: Instant,
    task: Option<Task<()>>,
}

impl StallMonitor {
    pub(crate) fn new(launched_at: Instant) -> Self {
        Self {
            launched_at,
            task: None,
        }
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool, cx: &mut App) {
        if enabled != self.task.is_some() {
            self.task = enabled.then(|| Self::spawn(self.launched_at, cx));
        }
    }

    fn spawn(launched_at: Instant, cx: &mut App) -> Task<()> {
        let mut due = Instant::now();
        cx.spawn(async move |cx| {
            loop {
                let late = Instant::now().saturating_duration_since(due);
                if (STALL..SUSPEND).contains(&late) {
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
    }
}
