use gpui::{
    Anchor, AnyElement, AppContext, Context, Entity, Focusable, FontWeight, InteractiveElement,
    IntoElement, ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, anchored,
    deferred, div, px, rgb,
};
use jayjay_core::{BisectPhase, BisectState, BisectVerdict};

use super::RepoWindow;
use crate::app::fonts::CodeText;
use crate::app::theme::{FONT_BODY, FONT_META, Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::overlay::{overlay_card, overlay_header, overlay_layer};
use crate::ui::primitives::{button, icon_button, inert_icon_button, small_button, text_tooltip};
use crate::ui::text_area::TextArea;

pub(crate) struct BisectSheet {
    good: Entity<TextArea>,
    bad: Entity<TextArea>,
    range: Entity<TextArea>,
    focus: Entity<TextArea>,
    focus_pending: bool,
}

impl RepoWindow {
    pub(crate) fn open_bisect_sheet(&mut self, bad: Option<String>, cx: &mut Context<Self>) {
        let bad_text = bad.unwrap_or_default();
        let from_here = !bad_text.is_empty();
        let good =
            cx.new(|cx| TextArea::new("trunk()", jayjay_i18n::repo::good_end(), false, 32., cx));
        let bad_input =
            cx.new(|cx| TextArea::new(bad_text, jayjay_i18n::repo::bad_end(), false, 32., cx));
        let range =
            cx.new(|cx| TextArea::new("", jayjay_i18n::repo::revset_range(), false, 32., cx));
        let focus = if from_here {
            good.clone()
        } else {
            bad_input.clone()
        };
        self.bisect_sheet = Some(BisectSheet {
            good,
            bad: bad_input,
            range,
            focus,
            focus_pending: true,
        });
        cx.notify();
    }

    pub(crate) fn close_bisect_sheet(&mut self, cx: &mut Context<Self>) {
        if self.bisect_sheet.take().is_some() {
            cx.notify();
        }
    }

    pub(crate) fn submit_bisect_sheet(&mut self, cx: &mut Context<Self>) {
        let Some(sheet) = self.bisect_sheet.as_ref() else {
            return;
        };
        let good = sheet.good.read(cx).text();
        let bad = sheet.bad.read(cx).text();
        let range = sheet.range.read(cx).text();
        let range = match jayjay_core::compose_bisect_range(&good, &bad, &range) {
            Ok(range) => range,
            Err(error) => {
                self.show_toast(crate::app::error_text(error), cx);
                return;
            }
        };
        self.bisect_sheet = None;
        self.vm.update(cx, |vm, cx| vm.start_bisect(range, cx));
    }

    pub(crate) fn mark_bisect(&mut self, verdict: BisectVerdict, cx: &mut Context<Self>) {
        let commit_id = self.vm.read(cx).bisect.state.as_ref().and_then(|state| {
            state
                .candidate
                .as_ref()
                .map(|candidate| candidate.commit_id.clone())
        });
        let Some(commit_id) = commit_id else {
            return;
        };
        self.vm
            .update(cx, |vm, cx| vm.mark_bisect(commit_id, verdict, cx));
    }

    pub(crate) fn run_bisect_command(&mut self, to_end: bool, cx: &mut Context<Self>) {
        self.bisect_test_runner_open = false;
        let command = self.bisect_command.read(cx).text();
        self.vm
            .update(cx, |vm, cx| vm.run_bisect(command, to_end, cx));
    }
}

pub(super) fn bisect_bar(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    let vm = view.vm.read(cx);
    let state = vm.bisect.state.clone()?;
    let refreshing = vm.loading.refreshing;
    let enabled = BarEnablement {
        mark: vm.bisect.can_mark() && !refreshing,
        act: !vm.bisect.is_busy() && !refreshing,
        stop: vm.bisect.can_stop() && (vm.bisect.command_is_running() || !refreshing),
    };
    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .child(icons::icon(glyph::ARROWS_LEFT_RIGHT, 13., t.bisect_accent))
        .child(
            div()
                .flex_none()
                .font_weight(FontWeight::SEMIBOLD)
                .child(state.phase.title()),
        )
        .child(
            div()
                .debug_selector(|| "bisect-status".to_owned())
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(rgb(t.fg_dim))
                .child(state.status.clone()),
        )
        .child(bisect_actions(view, &state, enabled, t, cx));
    let mut bar = div()
        .debug_selector(|| "bisect-bar".to_owned())
        .flex()
        .flex_col()
        .gap(px(4.))
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .bg(rgb(t.bisect_bar_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .text_size(ui_font_size(FONT_BODY))
        .text_color(rgb(t.fg))
        .child(header);
    if let Some(error) = state.command_error.clone() {
        bar = bar.child(
            div()
                .debug_selector(|| "bisect-command-error".to_owned())
                .text_size(ui_font_size(FONT_META))
                .text_color(rgb(t.tag_conflict_fg))
                .child(error),
        );
    }
    if let Some(result) = state.result.as_ref() {
        let kept = result
            .kept
            .iter()
            .map(|change| jayjay_i18n::repo::kept_change(subject(&change.description)));
        for note in result.restore_note.clone().into_iter().chain(kept) {
            bar = bar.child(
                div()
                    .text_size(ui_font_size(FONT_META))
                    .text_color(rgb(t.fg_dim))
                    .child(note),
            );
        }
    }
    Some(bar.into_any_element())
}

#[derive(Clone, Copy)]
struct BarEnablement {
    mark: bool,
    act: bool,
    stop: bool,
}

trait PhaseTitle {
    fn title(self) -> String;
}

impl PhaseTitle for BisectPhase {
    fn title(self) -> String {
        match self {
            BisectPhase::Evaluating => jayjay_i18n::repo::bisecting(),
            BisectPhase::PausedRangeChanged => jayjay_i18n::repo::bisect_paused(),
            BisectPhase::Finished => jayjay_i18n::repo::bisect_finished(),
        }
    }
}

fn subject(description: &str) -> &str {
    description.lines().next().unwrap_or("a change")
}

fn bisect_actions(
    view: &RepoWindow,
    state: &BisectState,
    enabled: BarEnablement,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let row = div()
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .gap(px(6.));
    let row = match state.phase {
        BisectPhase::Evaluating => row
            .child(mark_button(
                "bisect-good",
                jayjay_i18n::repo::good(),
                BisectVerdict::Good,
                enabled.mark,
                t,
                cx,
            ))
            .child(mark_button(
                "bisect-bad",
                jayjay_i18n::repo::bad(),
                BisectVerdict::Bad,
                enabled.mark,
                t,
                cx,
            ))
            .child(mark_button(
                "bisect-skip",
                jayjay_i18n::repo::skip(),
                BisectVerdict::Skip,
                enabled.mark,
                t,
                cx,
            ))
            .child(test_runner_button(view, state, enabled.act, t, cx))
            .child(stop_button(enabled.stop, t, cx)),
        BisectPhase::PausedRangeChanged => row
            .child(action_button(
                "bisect-restart",
                jayjay_i18n::repo::restart(),
                enabled.act,
                t,
                cx,
                |view, cx| {
                    view.vm.update(cx, |vm, cx| vm.restart_bisect(cx));
                },
            ))
            .child(stop_button(enabled.stop, t, cx)),
        BisectPhase::Finished => row.child(action_button(
            "bisect-dismiss",
            jayjay_i18n::repo::dismiss(),
            enabled.act,
            t,
            cx,
            |view, cx| {
                view.vm.update(cx, |vm, cx| vm.dismiss_bisect(cx));
            },
        )),
    };
    row.into_any_element()
}

fn test_runner_button(
    view: &RepoWindow,
    state: &BisectState,
    enabled: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut button = small_button("bisect-run-test", jayjay_i18n::repo::run_test_ellipsis(), t)
        .debug_selector(|| "bisect-run-test".to_owned());
    if enabled {
        button = button.on_click(cx.listener(|view, _, window, cx| {
            view.bisect_test_runner_open = !view.bisect_test_runner_open;
            if view.bisect_test_runner_open {
                window.focus(&view.bisect_command.read(cx).focus_handle(cx), cx);
            }
            cx.notify();
        }));
    } else {
        button = button.opacity(0.45);
    }
    let mut anchor = div().relative().child(button);
    if view.bisect_test_runner_open && enabled {
        anchor = anchor.child(
            div().absolute().top_full().right_0().child(deferred(
                anchored()
                    .anchor(Anchor::TopRight)
                    .snap_to_window_with_margin(px(6.))
                    .child(test_runner(view, state, t, cx)),
            )),
        );
    }
    anchor.into_any_element()
}

fn test_runner(
    view: &RepoWindow,
    state: &BisectState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut panel = div()
        .id("bisect-test-runner")
        .debug_selector(|| "bisect-test-runner".to_owned())
        .occlude()
        .mt(px(4.))
        .w(px(340.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(12.))
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.detail_bg))
        .shadow_md()
        .text_size(ui_font_size(FONT_META))
        .on_mouse_down_out(cx.listener(|view, _, _, cx| {
            view.bisect_test_runner_open = false;
            cx.notify();
        }))
        .child(
            div()
                .text_color(rgb(t.fg_dim))
                .child(jayjay_i18n::repo::exit_n0_marks_the_change_good()),
        )
        .child(view.bisect_command.clone())
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(6.))
                .child(action_button(
                    "bisect-run-step",
                    jayjay_i18n::repo::run_for_this_step(),
                    true,
                    t,
                    cx,
                    |view, cx| {
                        view.run_bisect_command(false, cx);
                    },
                ))
                .child(action_button(
                    "bisect-run-to-end",
                    jayjay_i18n::repo::run_to_the_end(),
                    true,
                    t,
                    cx,
                    |view, cx| {
                        view.run_bisect_command(true, cx);
                    },
                )),
        );
    if !state.command_output.is_empty() {
        panel = panel.child(
            div()
                .debug_selector(|| "bisect-output".to_owned())
                .code_text(11., t)
                .text_color(rgb(t.fg_dim))
                .child(state.command_output.clone()),
        );
    }
    panel.into_any_element()
}

fn stop_button(enabled: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let size = t.scaled_control_height(22., 11.);
    let button = if enabled {
        icon_button("bisect-stop", glyph::X_CIRCLE, 14., size, size, t.fg_dim, t).on_click(
            cx.listener(|view, _, _, cx| {
                view.vm.update(cx, |vm, cx| vm.stop_bisect(cx));
            }),
        )
    } else {
        inert_icon_button("bisect-stop", glyph::X_CIRCLE, 14., size, size, t.fg_dim).opacity(0.45)
    };
    button
        .debug_selector(|| "bisect-stop".to_owned())
        .tooltip(text_tooltip(
            jayjay_i18n::repo::stop_bisecting_and_restore_the_working(),
        ))
        .into_any_element()
}

fn mark_button(
    id: &'static str,
    label: impl Into<SharedString>,
    verdict: BisectVerdict,
    enabled: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    action_button(id, label, enabled, t, cx, move |view, cx| {
        view.mark_bisect(verdict, cx);
    })
}

fn action_button(
    id: &'static str,
    label: impl Into<SharedString>,
    enabled: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
    on_click: impl Fn(&mut RepoWindow, &mut Context<RepoWindow>) + 'static,
) -> AnyElement {
    let mut button = small_button(id, label, t).debug_selector(move || id.to_owned());
    if enabled {
        button = button.on_click(cx.listener(move |view, _, _, cx| on_click(view, cx)));
    } else {
        button = button.opacity(0.45);
    }
    button.into_any_element()
}

pub(super) fn bisect_sheet_overlay(
    sheet: &mut BisectSheet,
    t: &Theme,
    window: &mut Window,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    if sheet.focus_pending {
        sheet.focus_pending = false;
        window.focus(&sheet.focus.read(cx).focus_handle(cx), cx);
    }
    let good = sheet.good.clone();
    let bad = sheet.bad.clone();
    let range = sheet.range.clone();
    overlay_layer()
        .child(
            overlay_card(t, 460.)
                .debug_selector(|| "bisect-sheet".to_owned())
                .child(overlay_header(
                    glyph::ARROWS_LEFT_RIGHT,
                    t.fg_dim,
                    jayjay_i18n::repo::bisect(),
                    "",
                    t,
                ))
                .child(field_label(jayjay_i18n::repo::good(), t))
                .child(good)
                .child(field_label(jayjay_i18n::repo::bad(), t))
                .child(bad)
                .child(field_label(jayjay_i18n::repo::range(), t))
                .child(range)
                .child(
                    div()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(jayjay_i18n::repo::bisect_range_hint()),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            button(
                                "bisect-sheet-cancel",
                                jayjay_i18n::common::cancel(),
                                t,
                                false,
                            )
                            .debug_selector(|| "bisect-sheet-cancel".to_owned())
                            .on_click(cx.listener(|view, _, _, cx| view.close_bisect_sheet(cx))),
                        )
                        .child(
                            button("bisect-sheet-start", jayjay_i18n::repo::start(), t, true)
                                .debug_selector(|| "bisect-sheet-start".to_owned())
                                .on_click(
                                    cx.listener(|view, _, _, cx| view.submit_bisect_sheet(cx)),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn field_label(text: impl Into<SharedString>, t: &Theme) -> AnyElement {
    div()
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .child(text.into())
        .into_any_element()
}
