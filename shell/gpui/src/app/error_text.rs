use gpui::SharedString;
use jayjay_core::error_message::unwrap_command_error;

pub(crate) fn error_text(error: impl std::fmt::Display) -> SharedString {
    unwrap_command_error(&error.to_string()).into()
}
