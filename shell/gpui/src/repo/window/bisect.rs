use gpui::{
    AnyElement, AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::{BisectPhase, BisectState, BisectVerdict};

use super::RepoWindow;
use crate::app::theme::{FONT_META, Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::overlay::{overlay_card, overlay_header, overlay_layer};
use crate::ui::primitives::{button, small_button};
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
        let good = cx.new(|cx| TextArea::new("trunk()", "Good end", false, 32., cx));
        let bad_input = cx.new(|cx| TextArea::new(bad_text, "Bad end", false, 32., cx));
        let range = cx.new(|cx| TextArea::new("", "Revset range", false, 32., cx));
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
    let can_mark = vm.bisect.can_mark() && !refreshing;
    let can_act = !vm.bisect.is_busy() && !refreshing;
    let stop_enabled = vm.bisect.can_stop() && (vm.bisect.command_is_running() || !refreshing);
    let mut column = div()
        .debug_selector(|| "bisect-bar".to_owned())
        .flex()
        .flex_col()
        .gap(px(4.))
        .w_full()
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_t_1()
        .border_color(rgb(t.border))
        .text_size(ui_font_size(FONT_META))
        .text_color(rgb(t.fg))
        .child(
            div()
                .debug_selector(|| "bisect-status".to_owned())
                .text_color(rgb(t.fg_dim))
                .child(state.status.clone()),
        );
    column = column.child(bisect_actions(
        view,
        &state,
        can_mark,
        can_act,
        stop_enabled,
        t,
        cx,
    ));
    if let Some(error) = state.command_error.clone() {
        column = column.child(
            div()
                .debug_selector(|| "bisect-command-error".to_owned())
                .text_color(rgb(t.tag_conflict_fg))
                .child(error),
        );
    }
    if !state.command_output.is_empty() {
        column = column.child(
            div()
                .debug_selector(|| "bisect-output".to_owned())
                .text_color(rgb(t.fg_dim))
                .child(state.command_output.clone()),
        );
    }
    if let Some(result) = state.result.as_ref() {
        if let Some(note) = result.restore_note.clone() {
            column = column.child(div().text_color(rgb(t.fg_dim)).child(note));
        }
        for kept in &result.kept {
            column = column.child(
                div()
                    .text_color(rgb(t.fg_dim))
                    .child(format!("Kept {}", kept.description)),
            );
        }
    }
    Some(column.into_any_element())
}

fn bisect_actions(
    view: &RepoWindow,
    state: &BisectState,
    can_mark: bool,
    can_act: bool,
    stop_enabled: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut row = div().flex().flex_row().items_center().gap(px(6.));
    if state.phase == BisectPhase::Evaluating {
        row = row
            .child(mark_button(
                "bisect-good",
                "Good",
                BisectVerdict::Good,
                can_mark,
                t,
                cx,
            ))
            .child(mark_button(
                "bisect-bad",
                "Bad",
                BisectVerdict::Bad,
                can_mark,
                t,
                cx,
            ))
            .child(mark_button(
                "bisect-skip",
                "Skip",
                BisectVerdict::Skip,
                can_mark,
                t,
                cx,
            ))
            .child(div().w(px(220.)).child(view.bisect_command.clone()))
            .child(action_button(
                "bisect-run-step",
                "Run for this step",
                can_act,
                t,
                cx,
                |view, cx| {
                    view.run_bisect_command(false, cx);
                },
            ))
            .child(action_button(
                "bisect-run-to-end",
                "Run to the end",
                can_act,
                t,
                cx,
                |view, cx| {
                    view.run_bisect_command(true, cx);
                },
            ));
    }
    if state.phase == BisectPhase::PausedRangeChanged {
        row = row.child(action_button(
            "bisect-restart",
            "Restart",
            can_act,
            t,
            cx,
            |view, cx| {
                view.vm.update(cx, |vm, cx| vm.restart_bisect(cx));
            },
        ));
    }
    if state.phase == BisectPhase::Finished {
        row = row.child(action_button(
            "bisect-dismiss",
            "Dismiss",
            can_act,
            t,
            cx,
            |view, cx| {
                view.vm.update(cx, |vm, cx| vm.dismiss_bisect(cx));
            },
        ));
    } else {
        row = row.child(action_button(
            "bisect-stop",
            "Stop",
            stop_enabled,
            t,
            cx,
            |view, cx| {
                view.vm.update(cx, |vm, cx| vm.stop_bisect(cx));
            },
        ));
    }
    row.into_any_element()
}

fn mark_button(
    id: &'static str,
    label: &'static str,
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
    label: &'static str,
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
                    "Bisect",
                    "",
                    t,
                ))
                .child(field_label("Good", t))
                .child(good)
                .child(field_label("Bad", t))
                .child(bad)
                .child(field_label("Range", t))
                .child(range)
                .child(
                    div()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.fg_dim))
                        .child("A range is used as-is. Otherwise the ends become (good)..(bad)."),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            button("bisect-sheet-cancel", "Cancel", t, false)
                                .debug_selector(|| "bisect-sheet-cancel".to_owned())
                                .on_click(
                                    cx.listener(|view, _, _, cx| view.close_bisect_sheet(cx)),
                                ),
                        )
                        .child(
                            button("bisect-sheet-start", "Start", t, true)
                                .debug_selector(|| "bisect-sheet-start".to_owned())
                                .on_click(
                                    cx.listener(|view, _, _, cx| view.submit_bisect_sheet(cx)),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn field_label(text: &'static str, t: &Theme) -> AnyElement {
    div()
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .child(text)
        .into_any_element()
}
