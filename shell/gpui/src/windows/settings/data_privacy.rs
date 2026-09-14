use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, px};

use super::shared::{field_row, section_title, subsection_title};
use crate::app::config::{self, AppConfig};
use crate::app::theme::Theme;
use crate::ui::primitives::boolean_toggle_button;

pub(super) fn data_privacy_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    let active = cfg.telemetry.enabled;
    let toggle = boolean_toggle_button(
        SharedString::from("setting-privacy-telemetry"),
        active,
        t,
        move |_, _, cx| {
            let enabled = !active;
            config::update(cx, |c| c.telemetry.enabled = enabled);
            crate::app::telemetry::maybe_ping(enabled);
        },
    );

    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Data & Privacy", t))
        .child(subsection_title("Privacy", t))
        .child(field_row(
            "Share anonymous build and OS stats",
            toggle,
            "No repository, file, or command data is sent.",
            t,
        ))
        .into_any_element()
}
