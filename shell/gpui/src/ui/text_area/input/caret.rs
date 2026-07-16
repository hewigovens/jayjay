use gpui::{Context, Window};

use super::super::TextArea;

impl TextArea {
    pub(in crate::ui::text_area) fn ensure_focus_handlers(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.focus_subscriptions.is_empty() {
            return;
        }
        let focus_handle = self.focus_handle.clone();
        self.focus_subscriptions = vec![
            cx.on_focus(&focus_handle, window, |input, _window, cx| {
                input.show_caret(cx);
            }),
            cx.on_blur(&focus_handle, window, |input, _window, cx| {
                input.hide_caret(cx);
            }),
        ];
        if self.focus_handle.is_focused(window) {
            self.show_caret(cx);
        }
    }

    pub(in crate::ui::text_area) fn caret_visible(&self) -> bool {
        self.caret.visible()
    }

    pub(in crate::ui::text_area) fn show_caret(&mut self, cx: &mut Context<Self>) {
        if !self.is_editable() {
            self.caret.hide(cx);
            return;
        }
        // Every edit and caret move funnels through here — the one scroll-into-view trigger.
        self.scroll_caret_into_view = true;
        self.caret.show(cx, |input, generation, cx| {
            input.toggle_caret(generation, cx)
        });
    }

    fn hide_caret(&mut self, cx: &mut Context<Self>) {
        self.caret.hide(cx);
    }

    fn toggle_caret(&mut self, generation: u64, cx: &mut Context<Self>) -> bool {
        self.caret.toggle_if_current(generation, cx)
    }
}
