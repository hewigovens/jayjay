use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

use super::shared::{section_title, toggle_field};
use crate::app::theme::Theme;

pub(super) fn features_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(section_title("Features", t))
        .child(toggle_field(
            "Skip abandon confirmation",
            cfg.features.skip_abandon_confirmation,
            "Don't prompt before abandoning a change.",
            |c| c.features.skip_abandon_confirmation ^= true,
            "feat-aban",
            t,
        ))
        .child(toggle_field(
            "Confirm drag-rebase",
            cfg.features.confirm_drag_rebase,
            "Show a confirmation sheet for drag-to-rebase.",
            |c| c.features.confirm_drag_rebase ^= true,
            "feat-rebase",
            t,
        ))
        .child(toggle_field(
            "Send anonymous usage stats",
            cfg.telemetry.enabled,
            "A daily ping with app version, OS, and CPU arch. No personal data.",
            |c| c.telemetry.enabled ^= true,
            "feat-telemetry",
            t,
        ))
        .into_any_element()
}
