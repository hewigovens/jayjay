use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, Div, Entity, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
    relative, rgb,
};
use jayjay_core::converge::ConvergeReview;

use super::body::ConvergeBody;
use super::contents::Loaded;
use super::sheet::{ConvergePhase, ConvergeSheet};
use crate::app::actions::{MergeNextHunk, MergePreviousHunk, MergeUseLeftHunk, MergeUseRightHunk};
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::view_model::RepoViewModel;
use crate::repo::window::{RepoModal, RepoWindow};
use crate::ui::icons::glyph;
use crate::ui::overlay::{overlay_actions, overlay_card, overlay_header, overlay_layer};
use crate::ui::primitives::{button, icon_label};

/// Scroll children before the first choice card: notices, versions, and the choices heading.
pub(super) const FIRST_CARD_CHILD: usize = 3;

impl RepoWindow {
    pub(crate) fn converge_actions(&self, root: Div, cx: &mut Context<Self>) -> Div {
        if !matches!(self.modal, Some(RepoModal::Converge(_))) {
            return root;
        }
        root.key_context("ConvergeSheet MergeHunks")
            .on_action(cx.listener(|view, _: &MergeUseLeftHunk, _, cx| {
                view.update_converge_review(cx, |review| review.use_version_at(0));
            }))
            .on_action(cx.listener(|view, _: &MergeUseRightHunk, _, cx| {
                view.update_converge_review(cx, |review| review.use_version_at(1));
            }))
            .on_action(cx.listener(|view, _: &MergePreviousHunk, _, cx| {
                view.update_converge_review(cx, |review| review.move_focus(-1));
            }))
            .on_action(cx.listener(|view, _: &MergeNextHunk, _, cx| {
                view.update_converge_review(cx, |review| review.move_focus(1));
            }))
    }
}

pub(crate) fn converge_overlay(
    sheet: &mut ConvergeSheet,
    root_focus: &FocusHandle,
    vm: &Entity<RepoViewModel>,
    t: &Theme,
    window: &mut Window,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    sheet.take_focus(root_focus, window, cx);
    let sheet = &*sheet;
    let body = sheet
        .review
        .as_ref()
        .map(|review| ConvergeBody::new(sheet, review, &vm.read(cx).graph.changes, t));
    let scroll = div()
        .id("converge-scroll")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap(px(14.))
        .pr(px(4.))
        .overflow_y_scroll()
        .track_scroll(&sheet.scroll)
        .child(notices(sheet, t, cx))
        .children(body.iter().flat_map(|body| body.sections(cx)));
    let panel = overlay_card(t, 760.)
        .debug_selector(|| "converge-sheet".to_owned())
        .max_h(relative(0.9))
        .child(overlay_header(
            glyph::GIT_MERGE,
            t.fg_dim,
            jayjay_i18n::converge::title(),
            "",
            t,
        ))
        .children(body.as_ref().map(ConvergeBody::summary))
        .child(scroll)
        .child(footer(sheet, t, cx));
    overlay_layer().child(panel).into_any_element()
}

fn notices(sheet: &ConvergeSheet, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let notices = div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .text_size(ui_font_size(12.))
        .when(sheet.was_stale, |notices| {
            notices.child(
                icon_label(glyph::SYNC, jayjay_i18n::converge::stale(), 12., t.fg_dim)
                    .debug_selector(|| "converge-stale".to_owned())
                    .whitespace_normal(),
            )
        });
    match &sheet.phase {
        ConvergePhase::Loading => {
            notices.child(caption(jayjay_i18n::converge::loading(), t).text_size(ui_font_size(12.)))
        }
        ConvergePhase::Failed(error) => notices
            .child(error_text(error.clone(), t).debug_selector(|| "converge-error".to_owned()))
            .child(link(
                "converge-refresh",
                jayjay_i18n::converge::refresh(),
                true,
                t,
                cx,
                RepoWindow::load_converge_preview,
            )),
        ConvergePhase::Ready | ConvergePhase::Applying => notices,
    }
    .into_any_element()
}

fn footer(sheet: &ConvergeSheet, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let applying = matches!(sheet.phase, ConvergePhase::Applying);
    let resolves = sheet
        .review
        .as_ref()
        .is_some_and(ConvergeReview::will_conflict);
    let (label, selector) = match (applying, resolves) {
        (true, _) => (jayjay_i18n::converge::applying(), "converge-apply"),
        (false, true) => (
            jayjay_i18n::converge::apply_and_resolve(),
            "converge-apply-and-resolve",
        ),
        (false, false) => (jayjay_i18n::converge::apply(), "converge-apply"),
    };
    let cancel = button("converge-cancel", jayjay_i18n::common::cancel(), t, false)
        .debug_selector(|| "converge-cancel".to_owned());
    let cancel = if applying {
        cancel.opacity(0.45)
    } else {
        cancel.on_click(cx.listener(|view, _, _, cx| view.close_converge(cx)))
    };
    let apply =
        button("converge-apply", label, t, true).debug_selector(move || selector.to_owned());
    let apply = if sheet.can_apply() {
        apply.on_click(cx.listener(|view, _, _, cx| view.apply_converge(cx)))
    } else {
        apply.opacity(0.45)
    };
    let hint = sheet.review.as_ref().and_then(ConvergeReview::hint);
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .pt(px(10.))
        .border_t_1()
        .border_color(rgb(t.border))
        .children(hint.map(|hint| {
            icon_label(glyph::INFO, hint, 11., t.fg_dim)
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
        }))
        .child(div().flex_1())
        .child(overlay_actions(cancel, apply))
        .into_any_element()
}

pub(super) fn heading(text: String) -> Div {
    div()
        .text_size(ui_font_size(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text)
}

pub(super) fn card_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_none()
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.detail_bg))
}

pub(super) fn caption(text: impl Into<SharedString>, t: &Theme) -> Div {
    div()
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .child(text.into())
}

pub(super) fn error_text(error: SharedString, t: &Theme) -> Div {
    div()
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.error_fg))
        .whitespace_normal()
        .child(error)
}

pub(super) fn loaded<T>(
    slot: &Loaded<T>,
    t: &Theme,
    content: impl FnOnce(&Arc<T>) -> AnyElement,
) -> AnyElement {
    match slot {
        None => caption(jayjay_i18n::converge::loading(), t)
            .p(px(10.))
            .into_any_element(),
        Some(Err(error)) => error_text(error.clone(), t).p(px(10.)).into_any_element(),
        Some(Ok(value)) => content(value),
    }
}

/// Clicks stop propagating so a link inside a choice card does not also focus the card.
pub(super) fn link(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    enabled: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
    on_click: impl Fn(&mut RepoWindow, &mut Context<RepoWindow>) + 'static,
) -> Stateful<Div> {
    let id = id.into();
    let selector = id.to_string();
    let link = div()
        .id(id)
        .debug_selector(move || selector.clone())
        .flex()
        .items_center()
        .gap(px(4.))
        .text_size(ui_font_size(11.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(t.selected_accent))
        .child(label.into());
    if enabled {
        link.cursor_pointer()
            .hover(|style| style.underline())
            .on_click(cx.listener(move |view, _, _, cx| {
                cx.stop_propagation();
                on_click(view, cx);
            }))
    } else {
        link.opacity(0.45)
    }
}
