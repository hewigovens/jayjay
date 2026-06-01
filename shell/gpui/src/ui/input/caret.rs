use std::time::Duration;

use gpui::Context;

const CARET_BLINK_INTERVAL: Duration = Duration::from_millis(530);

#[derive(Debug, Default)]
pub struct CaretBlink {
    visible: bool,
    generation: u64,
}

impl CaretBlink {
    pub(crate) fn visible(&self) -> bool {
        self.visible
    }

    pub(crate) fn show<T>(
        &mut self,
        cx: &mut Context<T>,
        mut tick: impl FnMut(&mut T, u64, &mut Context<T>) -> bool + 'static,
    ) where
        T: 'static,
    {
        self.visible = true;
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        cx.notify();
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(CARET_BLINK_INTERVAL).await;
                let keep_going = this
                    .update(cx, |this, cx| tick(this, generation, cx))
                    .unwrap_or(false);
                if !keep_going {
                    break;
                }
            }
        })
        .detach();
    }

    pub(crate) fn hide<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.visible = false;
        self.generation = self.generation.wrapping_add(1);
        cx.notify();
    }

    pub(crate) fn toggle_if_current<T: 'static>(
        &mut self,
        generation: u64,
        cx: &mut Context<T>,
    ) -> bool {
        if self.generation != generation {
            return false;
        }
        self.visible = !self.visible;
        cx.notify();
        true
    }
}
