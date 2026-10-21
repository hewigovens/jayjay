use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_review::ReviewStoreSummary;

use super::SettingsView;
use super::shared::{field_row, form_section};
use crate::app::config::{self, AppConfig, TelemetryConfig};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::overlay::{confirmation_card, overlay_actions, overlay_layer};
use crate::ui::primitives::{boolean_toggle_button, button};

pub(super) fn data_privacy_section(
    view: &SettingsView,
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
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
        .gap(px(24.))
        .child(form_section(
            "Privacy",
            [field_row(
                glyph::CHART_BAR,
                TelemetryConfig::LABEL,
                toggle,
                t,
            )],
            t,
        ))
        .child(form_section(
            "Diagnostics",
            [field_row(
                glyph::WRENCH,
                "Debug mode",
                debug_mode_toggle(cfg, t),
                t,
            )],
            t,
        ))
        .child(form_section(
            "Review Data",
            [review_data_row(view.review_summary, t, cx)],
            t,
        ))
        .into_any_element()
}

fn debug_mode_toggle(cfg: &AppConfig, t: &Theme) -> AnyElement {
    let enabled = cfg.debug_mode();
    boolean_toggle_button(
        SharedString::from("setting-debug-mode"),
        enabled,
        t,
        move |_, _, cx| config::update(cx, |c| c.debug_mode = Some(!enabled)),
    )
}

fn review_data_row(
    summary: Option<ReviewStoreSummary>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let clearable = summary.is_some_and(|summary| summary.marks > 0 || summary.notes > 0);
    let summary = summary.map(summary_text).unwrap_or_default();
    field_row(
        glyph::CHECK,
        "Review marks and notes",
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .text_size(ui_font_size(12.))
                    .text_color(rgb(t.fg_dim))
                    .child(SharedString::from(summary)),
            )
            .child(clear_review_button(clearable, t, cx))
            .into_any_element(),
        t,
    )
}

fn clear_review_button(clearable: bool, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let clear = button("settings-clear-review-data", "Clear…", t, false)
        .debug_selector(|| "settings-clear-review-data".to_owned());
    if !clearable {
        return clear.opacity(0.45).cursor_default().into_any_element();
    }
    clear
        .on_click(cx.listener(|view, _, _, cx| {
            view.confirm_clear_review = true;
            cx.notify();
        }))
        .into_any_element()
}

pub(super) fn clear_review_confirmation(
    summary: Option<ReviewStoreSummary>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let message = format!(
        "Removes {} for every repository. This cannot be undone.",
        summary.map(summary_text).unwrap_or_default()
    );
    overlay_layer()
        .child(
            confirmation_card("Clear all review marks and notes?", message, t).child(
                overlay_actions(
                    button("settings-clear-review-cancel", "Cancel", t, false)
                        .debug_selector(|| "settings-clear-review-cancel".to_owned())
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.confirm_clear_review = false;
                            cx.notify();
                        })),
                    button("settings-clear-review-submit", "Clear", t, true)
                        .debug_selector(|| "settings-clear-review-submit".to_owned())
                        .on_click(cx.listener(|view, _, _, cx| view.clear_review_data(cx))),
                ),
            ),
        )
        .into_any_element()
}

impl SettingsView {
    pub(super) fn clear_review_data(&mut self, cx: &mut Context<Self>) {
        let store = crate::repo::window::shared_review_store(cx);
        crate::repo::window::mutate_review_store(&store, jayjay_review::ReviewStore::clear_all);
        self.review_summary = Some(ReviewStoreSummary { marks: 0, notes: 0 });
        self.confirm_clear_review = false;
        // Open repo windows re-read the shared store on their next redraw, which nothing else would prompt.
        cx.refresh_windows();
        cx.notify();
    }
}

fn summary_text(summary: ReviewStoreSummary) -> String {
    let marks = if summary.marks == 1 { "mark" } else { "marks" };
    let notes = if summary.notes == 1 { "note" } else { "notes" };
    format!("{} {}, {} {}", summary.marks, marks, summary.notes, notes)
}
