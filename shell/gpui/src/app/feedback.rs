use gpui::{AnyWindowHandle, App, AppContext, ClipboardItem, PromptLevel};

use super::links::{FEEDBACK_ADDRESS, FEEDBACK_URL};

const OPEN_FAILURE_MESSAGE: &str = "Couldn’t Open Your Email App";

pub fn open(cx: &mut App) {
    let window = cx.active_window();
    cx.spawn(async move |cx| {
        let opened = cx
            .background_spawn(async { crate::platform::open_url(FEEDBACK_URL) })
            .await;
        if !opened {
            cx.update(|cx| present_open_failure(window, cx));
        }
    })
    .detach();
}

fn present_open_failure(window: Option<AnyWindowHandle>, cx: &mut App) {
    let Some(window) = window else {
        eprintln!("[jayjay-gpui] could not open email app; feedback address: {FEEDBACK_ADDRESS}");
        return;
    };
    if window
        .update(cx, |_, window, cx| {
            let answer = window.prompt(
                PromptLevel::Warning,
                OPEN_FAILURE_MESSAGE,
                Some(&format!("Send feedback directly to {FEEDBACK_ADDRESS}.")),
                &["Copy Address", "OK"],
                cx,
            );
            cx.spawn(async move |cx| {
                if answer.await == Ok(0) {
                    cx.update(|cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            FEEDBACK_ADDRESS.to_owned(),
                        ))
                    });
                }
            })
            .detach();
        })
        .is_err()
    {
        eprintln!(
            "[jayjay-gpui] could not present email fallback; feedback address: {FEEDBACK_ADDRESS}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn failed_open_shows_address_and_can_copy_it(cx: &mut gpui::TestAppContext) {
        let window = cx.add_window(|_, _| gpui::Empty);
        cx.update(|cx| present_open_failure(Some(window.into()), cx));

        let (message, detail) = cx.pending_prompt().expect("feedback fallback prompt");
        assert_eq!(message, OPEN_FAILURE_MESSAGE);
        assert!(detail.contains(FEEDBACK_ADDRESS));

        cx.simulate_prompt_answer("Copy Address");
        cx.run_until_parked();
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some(FEEDBACK_ADDRESS.to_owned())
        );
    }
}
