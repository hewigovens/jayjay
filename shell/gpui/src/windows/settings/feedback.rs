use gpui::{Context, SharedString};

use super::SettingsView;

impl SettingsView {
    pub(super) fn mark_copied(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.recently_copied = Some(id.clone());
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.recently_copied.as_ref() == Some(&id) {
                    view.recently_copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }
}
