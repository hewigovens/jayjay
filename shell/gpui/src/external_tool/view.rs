use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui::{
    App, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder as _, px, rgb,
};

use crate::app::theme::{theme, ui_font_size};
use crate::ui::icons::{glyph, icon};
use crate::ui::loading::loading_status;
use crate::ui::merge_scroll::MergeSync;
use crate::ui::primitives::button;
use crate::ui::text_area::TextArea;
use crate::ui::title_bar;

use super::ExternalToolInvocation;
use super::diff::ExternalDiffSession;
use super::merge::ExternalMergeSession;

pub(super) enum ExternalToolState {
    Loading,
    Diff(ExternalDiffSession),
    Merge {
        session: Box<ExternalMergeSession>,
        sources: [Entity<TextArea>; 3],
        result: Entity<TextArea>,
    },
    Error,
}

#[derive(Clone, Default)]
pub(super) struct ExternalToolExitState(Arc<AtomicBool>);

impl ExternalToolExitState {
    pub(super) fn fail(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub(super) fn code(&self, cancel_code: i32) -> i32 {
        if self.0.load(Ordering::Relaxed) {
            1
        } else {
            cancel_code
        }
    }
}

pub struct ExternalToolWindow {
    pub(super) invocation: ExternalToolInvocation,
    pub(super) state: ExternalToolState,
    pub(super) focus_handle: FocusHandle,
    pub(super) error_message: Option<String>,
    pub(super) show_merge_base: bool,
    pub(super) show_merge_raw: bool,
    pub(super) selected_merge_hunk: usize,
    pub(super) merge: MergeSync,
    pub(super) exit_state: ExternalToolExitState,
    /// How the tool leaves the process once jj's contract is met; tests substitute a recorder so the run survives a successful save.
    pub(super) exit: Rc<dyn Fn(i32)>,
}

impl ExternalToolWindow {
    pub fn new(invocation: ExternalToolInvocation, cx: &mut Context<Self>) -> Self {
        Self::with_exit(invocation, |code| std::process::exit(code), cx)
    }

    pub fn with_exit(
        invocation: ExternalToolInvocation,
        exit: impl Fn(i32) + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut window = Self {
            invocation,
            state: ExternalToolState::Loading,
            focus_handle: cx.focus_handle(),
            error_message: None,
            show_merge_base: false,
            show_merge_raw: false,
            selected_merge_hunk: 0,
            merge: MergeSync::default(),
            exit_state: ExternalToolExitState::default(),
            exit: Rc::new(exit),
        };
        window.load(cx);
        window
    }

    pub fn close_exit_code(&self) -> i32 {
        self.exit_state.code(self.invocation.cancel_exit_code())
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let status = {
            match &self.state {
                ExternalToolState::Loading => jayjay_i18n::repo::loading_ellipsis(),
                ExternalToolState::Diff(session) => {
                    jayjay_i18n::merge::changed_files(session.files.len())
                }
                ExternalToolState::Merge {
                    session, result, ..
                } => {
                    let count = session.unresolved_count(&result.read(cx).text());
                    if !session.is_text {
                        jayjay_i18n::merge::non_text_conflict()
                    } else if count == 0 && session.can_save(&result.read(cx).text()) {
                        jayjay_i18n::merge::ready_to_save()
                    } else if count == 0 {
                        jayjay_i18n::merge::unresolved_conflict_markers()
                    } else {
                        jayjay_i18n::merge::unresolved_blocks(count)
                    }
                }
                ExternalToolState::Error => jayjay_i18n::merge::could_not_open_tool_session(),
            }
        };
        let editable = !matches!(
            &self.invocation,
            ExternalToolInvocation::Diff {
                editable: false,
                ..
            }
        );
        let can_save = match &self.state {
            ExternalToolState::Diff(session) => session.editable,
            ExternalToolState::Merge {
                session, result, ..
            } => session.can_save(&result.read(cx).text()),
            ExternalToolState::Loading | ExternalToolState::Error => false,
        };
        let cancel_label = if editable {
            jayjay_i18n::common::cancel()
        } else {
            jayjay_i18n::common::close()
        };
        let (header_glyph, header_color) = match &self.invocation {
            ExternalToolInvocation::Diff { editable: true, .. } => {
                (glyph::PENCIL, t.selected_accent)
            }
            ExternalToolInvocation::Diff { .. } => (glyph::COLUMNS, t.selected_accent),
            ExternalToolInvocation::Merge { .. } => (glyph::GIT_MERGE, t.compare_accent),
        };
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.))
            .h(px(48.))
            .px(px(14.))
            .bg(rgb(t.header_bg))
            .border_b_1()
            .border_color(rgb(t.border))
            .child(icon(header_glyph, 16., header_color))
            .child(
                div()
                    .text_size(ui_font_size(14.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(self.invocation.title()),
            )
            .child(
                div()
                    .text_size(ui_font_size(11.))
                    .text_color(rgb(t.fg_dim))
                    .child(status),
            )
            .child(div().flex_1())
            .child(
                button("external-cancel", cancel_label, t, false)
                    .debug_selector(|| "external-cancel".to_owned())
                    .on_click(cx.listener(|view, _, _, _| (view.exit)(view.close_exit_code()))),
            );
        if editable && can_save {
            let label = match &self.state {
                ExternalToolState::Merge {
                    session, result, ..
                } if session.unresolved_count(&result.read(cx).text()) > 0 => {
                    jayjay_i18n::repo::save_partial()
                }
                _ => jayjay_i18n::repo::done(),
            };
            header = header.child(
                button("external-save", label, t, true)
                    .debug_selector(|| "external-save".to_owned())
                    .on_click(cx.listener(|view, _, _, cx| view.save(cx))),
            );
        } else if matches!(&self.state, ExternalToolState::Merge { .. }) {
            let message = match &self.state {
                ExternalToolState::Merge {
                    session, result, ..
                } if session.is_text_merge()
                    && session.unresolved_count(&result.read(cx).text()) > 0 =>
                {
                    jayjay_i18n::merge::resolve_all_conflict_blocks_before_saving()
                }
                _ => jayjay_i18n::merge::choose_a_side_to_save_this(),
            };
            header = header.child(
                div()
                    .text_size(ui_font_size(11.))
                    .text_color(rgb(t.fg_dim))
                    .child(message),
            );
        }
        header
    }
}

impl Focusable for ExternalToolWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ExternalToolWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let mut root = div()
            .font_family(crate::app::fonts::interface())
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .when(title_bar::DRAWN, |root| {
                root.child(title_bar::strip("external-tool-title-bar", &t, window))
            })
            .child(self.header(cx));
        if let Some(error) = &self.error_message {
            root = root.child(
                div()
                    .px(px(14.))
                    .py(px(8.))
                    .bg(rgb(t.tag_conflict_bg))
                    .text_color(rgb(t.error_fg))
                    .text_size(ui_font_size(12.))
                    .child(error.clone()),
            );
        }
        root.child(match &self.state {
            ExternalToolState::Loading => loading_status(
                match self.invocation {
                    ExternalToolInvocation::Diff { .. } => {
                        jayjay_i18n::merge::loading_comparison_ellipsis()
                    }
                    ExternalToolInvocation::Merge { .. } => {
                        jayjay_i18n::merge::loading_conflict_sides_ellipsis()
                    }
                },
                &t,
            ),
            ExternalToolState::Diff(_) => self.render_diff(&t, cx),
            ExternalToolState::Merge { .. } => self.render_merge(&t, cx),
            ExternalToolState::Error => div().flex_1().into_any_element(),
        })
    }
}
