use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::DiffEditDestination;

use super::RepoWindow;
use super::diff_edit_cards::diff_edit_body;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::{button, divider_h};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffEditSnapshot {
    pub active: bool,
    pub working_copy: bool,
    pub description: String,
    pub destinations: Vec<DiffEditDestination>,
    pub selected_files: usize,
    pub selected_lines: usize,
}

pub(super) fn diff_edit_view(
    view: &mut RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let header = header(view, t, cx);
    let body = diff_edit_body(view, t, cx);
    let action_bar = action_bar(view, t, cx);
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .child(header)
        .child(divider_h(t))
        .child(body)
        .child(action_bar)
        .into_any_element()
}

fn header(view: &RepoWindow, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let rev = view.diff_edit.change_id.as_deref().unwrap_or_default();
    let rev = rev.chars().take(12).collect::<String>();
    let summary = view.diff_edit_selection_text();
    let selecting = view.diff_edit_selecting_all();
    let deselect = view.diff_edit_should_deselect();
    let toggle_label = if selecting {
        "Selecting..."
    } else if deselect {
        "Deselect All"
    } else {
        "Select All"
    };
    let mut toggle = button("diff-edit-select-all", toggle_label, t, false);
    if !selecting {
        toggle = toggle.on_click(cx.listener(|view, _, _, cx| view.toggle_diff_edit_all(cx)));
    }
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(18.))
        .py(px(12.))
        .bg(rgb(t.header_bg))
        .child(
            div()
                .text_size(px(15.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("Diff Edit"),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(t.fg_dim))
                .child(rev),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(summary),
        )
        .child(toggle)
        .child(
            button("diff-edit-cancel", "Cancel", t, false)
                .debug_selector(|| "diff-edit-cancel".to_owned())
                .on_click(cx.listener(|view, _: &ClickEvent, _, cx| view.exit_diff_edit(cx))),
        )
        .into_any_element()
}

fn action_bar(view: &RepoWindow, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let working_copy = view.diff_edit.working_copy;
    let summary = view.diff_edit_selection_text();
    let mut row = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(18.))
        .py(px(10.))
        .bg(rgb(t.header_bg))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(summary),
        )
        .child(div().flex_1());
    if !working_copy {
        let description = view
            .diff_edit
            .message
            .lines()
            .next()
            .filter(|line| !line.trim().is_empty())
            .unwrap_or("Add description...")
            .to_owned();
        row = row
            .child(
                div()
                    .id("diff-edit-description")
                    .debug_selector(|| "diff-edit-description".to_owned())
                    .w(px(260.))
                    .h(px(28.))
                    .px(px(7.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .rounded_sm()
                    .bg(rgb(t.toggle_inactive_bg))
                    .text_color(rgb(t.toggle_inactive_fg))
                    .text_size(px(11.))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(t.row_alt_bg)))
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.open_diff_edit_description(cx);
                    }))
                    .child(icon(glyph::PENCIL_CIRCLE, 13., t.fg_dim))
                    .child(description),
            )
            .child(destination_button(
                "diff-edit-child",
                "Create New Child Change",
                DiffEditDestination::NewChild,
                true,
                t,
                cx,
            ))
            .child(destination_button(
                "diff-edit-parallel",
                "Create Parallel Change",
                DiffEditDestination::NewParallel,
                false,
                t,
                cx,
            ))
            .child(destination_button(
                "diff-edit-move",
                "Move to Working Copy",
                DiffEditDestination::MoveToWorkingCopy,
                false,
                t,
                cx,
            ));
    }
    row = row.child(destination_button(
        "diff-edit-done",
        "Done",
        DiffEditDestination::RemoveFromSource,
        false,
        t,
        cx,
    ));
    div()
        .flex()
        .flex_col()
        .child(divider_h(t))
        .child(row)
        .into_any_element()
}

fn destination_button(
    id: &'static str,
    label: &'static str,
    destination: DiffEditDestination,
    primary: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button(id, label, t, primary)
        .on_click(cx.listener(move |view, _, _, cx| view.start_diff_edit_apply(destination, cx)))
        .into_any_element()
}
