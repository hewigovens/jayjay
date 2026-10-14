use gpui::{AnyElement, IntoElement, SharedString};

use super::shared::{field_row, form_section};
use crate::app::config::{self, AppConfig, TelemetryConfig};
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
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

    form_section(
        "Privacy",
        [field_row(
            glyph::CHART_BAR,
            TelemetryConfig::LABEL,
            toggle,
            t,
        )],
        t,
    )
    .into_any_element()
}
