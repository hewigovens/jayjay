use std::path::Path;

use gpui::{AnyWindowHandle, App, AppContext, ClipboardItem, PromptLevel};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use super::config;
use super::links::{FEEDBACK_ADDRESS, FEEDBACK_URL, url_opener};

const OPEN_FAILURE_MESSAGE: &str = "Couldn’t Open Your Email App";

/// `mailto:` cannot carry attachments, so debug mode asks for the revealed log in the body instead.
fn feedback_url(log: Option<&Path>) -> String {
    let Some(log) = log else {
        return FEEDBACK_URL.to_owned();
    };
    let body = format!("Please attach the debug log: {}\n\n", log.display());
    format!(
        "{FEEDBACK_URL}&body={}",
        utf8_percent_encode(&body, NON_ALPHANUMERIC)
    )
}

pub(crate) fn open(cx: &mut App) {
    let window = cx.active_window();
    let open_url = url_opener(cx);
    let log = config::current(cx)
        .debug_mode()
        .then(jayjay_core::LogFile::path)
        .flatten();
    cx.spawn(async move |cx| {
        if let Some(log) = log.clone() {
            cx.background_spawn(async move { crate::platform::reveal_path(&log) })
                .detach();
        }
        let opened = cx
            .background_spawn(async move { open_url(&feedback_url(log.as_deref())) })
            .await;
        if !opened {
            cx.update(|cx| present_open_failure(window, cx));
        }
    })
    .detach();
}

fn present_open_failure(window: Option<AnyWindowHandle>, cx: &mut App) {
    let Some(window) = window else {
        log::warn!("could not open email app; feedback address: {FEEDBACK_ADDRESS}");
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
        log::warn!("could not present email fallback; feedback address: {FEEDBACK_ADDRESS}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_feedback_asks_for_the_log() {
        assert_eq!(feedback_url(None), FEEDBACK_URL);
        let url = feedback_url(Some(Path::new("/logs/jayjay.log")));
        assert!(url.starts_with(FEEDBACK_URL));
        assert!(url.contains("%2Flogs%2Fjayjay%2Elog"));
    }

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
