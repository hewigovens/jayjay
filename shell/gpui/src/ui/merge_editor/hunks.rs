use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, Role, SharedString, Stateful,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::diff::{DiffLine, DiffSpanStyle, FileDiff};
use jayjay_core::{MergeEditorHunk, MergeHunkSource};

use crate::app::theme::Theme;
use crate::diff::line::{ROW_HEIGHT, content_row, line_bg_color};
use crate::ui::primitives::text_tooltip;

fn merge_hunk_action_link(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    help: &'static str,
    enabled: bool,
    t: &Theme,
) -> Stateful<Div> {
    let id = id.into();
    let selector = id.to_string();
    let label = label.into();
    let mut action = div()
        .id(id)
        .debug_selector(move || selector.clone())
        .focusable()
        .tab_stop(enabled)
        .role(Role::Button)
        .aria_label(help)
        .flex()
        .items_center()
        .h(px(20.))
        .px(px(5.))
        .rounded_sm()
        .text_size(px(10.))
        .text_color(rgb(t.diff_text_dim))
        .child(label);
    if enabled {
        action = action
            .cursor_pointer()
            .hover(|style| style.bg(rgb(t.row_alt_bg)).underline())
            .tooltip(text_tooltip(help));
    } else {
        action = action.opacity(0.45);
    }
    action
}

pub(crate) fn merge_hunk_action_links(
    id_prefix: &'static str,
    index: usize,
    enabled: bool,
    t: &Theme,
) -> [(MergeHunkSource, Stateful<Div>); 3] {
    [
        (
            MergeHunkSource::Base,
            "Accept Base",
            "",
            "Use Base for this conflict",
        ),
        (
            MergeHunkSource::Left,
            "Accept Left",
            "  ⌥←",
            "Use Left for this conflict",
        ),
        (
            MergeHunkSource::Right,
            "Accept Right",
            "  ⌥→",
            "Use Right for this conflict",
        ),
    ]
    .map(|(source, label, shortcut, help)| {
        (
            source,
            merge_hunk_action_link(
                format!("{id_prefix}-hunk-{index}-{label}"),
                format!("{label}{shortcut}"),
                help,
                enabled,
                t,
            ),
        )
    })
}

pub(crate) fn merge_hunk_list_container(
    id: impl Into<SharedString>,
    cards: impl IntoIterator<Item = Stateful<Div>>,
) -> Stateful<Div> {
    let id = id.into();
    let selector = id.to_string();
    div()
        .id(id)
        .debug_selector(move || selector.clone())
        .key_context("MergeHunks")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap(px(12.))
        .p(px(12.))
        .overflow_y_scroll()
        .children(cards)
}

pub(crate) fn merge_hunk_card(
    hunk: &MergeEditorHunk,
    unified: &FileDiff,
    selected: bool,
    unresolved: bool,
    actions: impl IntoIterator<Item = AnyElement>,
    t: &Theme,
) -> Stateful<Div> {
    let diff_height = ROW_HEIGHT * unified.lines.len().max(1) as f32;
    let selector = format!("merge-hunk-{}", hunk.index);
    div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector.clone())
        .flex()
        .flex_none()
        .flex_col()
        .w_full()
        .rounded_md()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(if selected {
            t.selected_accent
        } else {
            t.border
        }))
        .opacity(if unresolved { 1. } else { 0.58 })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(34.))
                .px(px(10.))
                .bg(rgb(t.header_bg))
                .child(
                    div()
                        .text_size(px(11.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(format!("Conflict {}", hunk.index + 1)),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(if unresolved {
                            t.tag_conflict_fg
                        } else {
                            t.success_fg
                        }))
                        .child(if unresolved { "Unresolved" } else { "Resolved" }),
                )
                .child(div().flex_1()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(30.))
                .px(px(8.))
                .border_t_1()
                .border_b_1()
                .border_color(rgb(t.border))
                .children(actions)
                .child(div().flex_1())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .font_family(crate::app::fonts::mono())
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_faint))
                        .child("− Left")
                        .child("+ Right"),
                ),
        )
        .child(
            div()
                .id(SharedString::from(format!(
                    "merge-hunk-{}-unified-scroll",
                    hunk.index
                )))
                .h(px(diff_height))
                .overflow_x_scroll()
                .children(
                    unified
                        .lines
                        .iter()
                        .map(|line| merge_hunk_diff_line(line, t)),
                ),
        )
}

fn merge_hunk_diff_line(line: &DiffLine, t: &Theme) -> AnyElement {
    let marker = match line.style {
        DiffSpanStyle::Added => "+",
        DiffSpanStyle::Removed => "−",
        _ => "",
    };
    let bg = line_bg_color(line.style, line.conflict_kind, t);
    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(ROW_HEIGHT))
        .child(
            div()
                .flex_none()
                .w(px(24.))
                .h(px(ROW_HEIGHT))
                .bg(rgb(bg))
                .border_r_1()
                .border_color(rgb(t.border))
                .font_family(crate::app::fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(match line.style {
                    DiffSpanStyle::Added => t.diff_text_added,
                    DiffSpanStyle::Removed => t.diff_text_removed,
                    _ => t.fg_dim,
                }))
                .text_center()
                .child(marker),
        )
        .child(content_row(line, t, None, None, px(7.2)))
        .into_any_element()
}
