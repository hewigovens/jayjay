use gpui::{Context, FocusHandle, Subscription, Window};

use super::{LineInput, LineInputSelector};

impl LineInput {
    pub fn show_for_owner<T>(owner: &mut T, cx: &mut Context<T>, select: LineInputSelector<T>)
    where
        T: 'static,
    {
        if let Some(input) = select(owner) {
            input.reveal_cursor_edge();
            input.show_caret(cx, move |owner, generation, cx| {
                select(owner).is_some_and(|input| input.toggle_caret(generation, cx))
            });
        }
    }

    pub fn hide_for_owner<T>(owner: &mut T, cx: &mut Context<T>, select: LineInputSelector<T>)
    where
        T: 'static,
    {
        if let Some(input) = select(owner) {
            input.hide_caret(cx);
        }
    }

    pub fn install_focus_handlers<T>(
        owner: &mut T,
        focus_handle: &FocusHandle,
        subscriptions: &mut Vec<Subscription>,
        window: &mut Window,
        cx: &mut Context<T>,
        select: LineInputSelector<T>,
    ) where
        T: 'static,
    {
        if !subscriptions.is_empty() {
            return;
        }
        subscriptions.extend([
            cx.on_focus(focus_handle, window, move |owner, _window, cx| {
                Self::show_for_owner(owner, cx, select);
            }),
            cx.on_blur(focus_handle, window, move |owner, _window, cx| {
                Self::hide_for_owner(owner, cx, select);
            }),
        ]);
        if focus_handle.is_focused(window) {
            Self::show_for_owner(owner, cx, select);
        }
    }
}
