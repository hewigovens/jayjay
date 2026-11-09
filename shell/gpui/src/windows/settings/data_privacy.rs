use gpui::{
    AnyElement, App, AppContext, ClickEvent, Context, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::{JayResult, ReviewPrune};
use jayjay_i18n::{common, settings, telemetry};
use jayjay_review::{ReviewStore, ReviewStoreSummary};

use super::SettingsView;
use super::shared::{field_row, form_section};
use crate::app::config::{self, AppConfig};
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
            settings::privacy(),
            [field_row(glyph::CHART_BAR, telemetry::opt_in(), toggle, t)],
            t,
        ))
        .child(form_section(
            settings::diagnostics(),
            [field_row(
                glyph::WRENCH,
                settings::debug_mode(),
                debug_mode_toggle(cfg, t),
                t,
            )],
            t,
        ))
        .child(form_section(
            settings::review_data(),
            [review_data_row(view, t, cx)],
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

fn review_data_row(view: &SettingsView, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let has_data = view
        .review_summary
        .is_some_and(|summary| !summary.is_empty());
    let summary = view.review_summary.map(summary_text).unwrap_or_default();
    field_row(
        glyph::CHECK,
        settings::review_marks_and_notes(),
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
            .child(review_data_button(
                "settings-prune-review-data",
                settings::prune_review(),
                has_data && !view.checking_prune,
                t,
                cx.listener(|view, _, _, cx| view.check_prune(cx)),
            ))
            .child(review_data_button(
                "settings-clear-review-data",
                common::clear_ellipsis(),
                has_data,
                t,
                cx.listener(|view, _, _, cx| {
                    view.review_prompt = Some(ReviewDataPrompt::Clear);
                    cx.notify();
                }),
            ))
            .into_any_element(),
        t,
    )
}

fn review_data_button(
    id: &'static str,
    label: String,
    enabled: bool,
    t: &Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let button = button(id, label, t, false).debug_selector(move || id.to_owned());
    if !enabled {
        return button.opacity(0.45).cursor_default().into_any_element();
    }
    button.on_click(on_click).into_any_element()
}

pub(super) enum ReviewDataPrompt {
    Clear,
    Prune(ReviewStoreSummary),
    NothingToPrune,
    PruneFailed(String),
}

impl ReviewDataPrompt {
    pub(super) fn render(
        &self,
        summary: Option<ReviewStoreSummary>,
        t: &Theme,
        cx: &mut Context<SettingsView>,
    ) -> AnyElement {
        let (title, message) = match self {
            Self::Clear => (
                settings::clear_review_title(),
                settings::clear_review_message(&summary.map(summary_text).unwrap_or_default()),
            ),
            Self::Prune(removable) => (
                settings::prune_review_title(),
                settings::prune_review_message(&summary_text(*removable)),
            ),
            Self::NothingToPrune => (
                settings::prune_review_nothing_title(),
                settings::prune_review_nothing(),
            ),
            Self::PruneFailed(error) => (settings::prune_review_failed_title(), error.clone()),
        };
        let dismiss = |label| {
            button("settings-review-prompt-cancel", label, t, false)
                .debug_selector(|| "settings-review-prompt-cancel".to_owned())
                .on_click(cx.listener(|view, _, _, cx| {
                    view.review_prompt = None;
                    cx.notify();
                }))
        };
        let actions = match self {
            Self::Clear => overlay_actions(
                dismiss(common::cancel()),
                button("settings-review-prompt-submit", common::clear(), t, true)
                    .debug_selector(|| "settings-review-prompt-submit".to_owned())
                    .on_click(cx.listener(|view, _, _, cx| view.clear_review_data(cx))),
            ),
            Self::Prune(_) => overlay_actions(
                dismiss(common::cancel()),
                button(
                    "settings-review-prompt-submit",
                    settings::prune_review_confirm(),
                    t,
                    true,
                )
                .debug_selector(|| "settings-review-prompt-submit".to_owned())
                .on_click(cx.listener(|view, _, _, cx| view.prune_review_data(cx))),
            ),
            Self::NothingToPrune | Self::PruneFailed(_) => {
                div().flex().justify_end().child(dismiss(common::close()))
            }
        };
        overlay_layer()
            .child(confirmation_card(title, message, t).child(actions))
            .into_any_element()
    }
}

impl SettingsView {
    pub(super) fn clear_review_data(&mut self, cx: &mut Context<Self>) {
        let store = crate::repo::window::shared_review_store(cx);
        crate::repo::window::mutate_review_store(&store, ReviewStore::clear_all);
        self.review_summary = Some(ReviewStoreSummary { marks: 0, notes: 0 });
        self.review_prompt = None;
        // Open repo windows re-read the shared store on their next redraw, which nothing else would prompt.
        cx.refresh_windows();
        cx.notify();
    }

    fn check_prune(&mut self, cx: &mut Context<Self>) {
        self.checking_prune = true;
        cx.notify();
        self.with_review_prune(cx, |view, prune, cx| {
            view.review_prompt = Some(match prune {
                Ok(prune) => {
                    let store = crate::repo::window::shared_review_store(cx);
                    let mut store = store.borrow_mut();
                    store.refresh_if_stale();
                    let removable = prune.preview(&store);
                    if removable.is_empty() {
                        ReviewDataPrompt::NothingToPrune
                    } else {
                        ReviewDataPrompt::Prune(removable)
                    }
                }
                Err(error) => ReviewDataPrompt::PruneFailed(error.to_string()),
            });
        });
    }

    /// Recomputes the kept changes, since a change created while the prompt was up must keep its marks.
    fn prune_review_data(&mut self, cx: &mut Context<Self>) {
        self.review_prompt = None;
        self.checking_prune = true;
        cx.notify();
        self.with_review_prune(cx, |view, prune, cx| {
            let store = crate::repo::window::shared_review_store(cx);
            let pruned = prune.and_then(|prune| {
                let mut store = store.borrow_mut();
                prune.apply(&mut store).map(|_| store.summary())
            });
            match pruned {
                Ok(remaining) => view.review_summary = Some(remaining),
                Err(error) => {
                    view.review_prompt = Some(ReviewDataPrompt::PruneFailed(error.to_string()))
                }
            }
            // Open repo windows re-read the shared store on their next redraw, which nothing else would prompt.
            cx.refresh_windows();
        });
    }

    /// Opening every repository can take a while, SSH ones especially, so it runs off the main thread.
    fn with_review_prune(
        &mut self,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, JayResult<ReviewPrune>, &mut Context<Self>) + 'static,
    ) {
        // Open windows can belong to repositories that dropped off the recent list.
        let mut paths = config::current(cx).recent_repos.clone();
        paths.extend(crate::repo::window::repo_window_paths(None, cx));
        cx.spawn(async move |this, cx| {
            let prune = cx
                .background_spawn(async move { ReviewPrune::for_repositories(&paths) })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.checking_prune = false;
                then(view, prune, cx);
                cx.notify();
            });
        })
        .detach();
    }
}

fn summary_text(summary: ReviewStoreSummary) -> String {
    settings::review_summary(summary.marks, summary.notes)
}
