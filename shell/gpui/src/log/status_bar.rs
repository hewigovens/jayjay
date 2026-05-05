use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::LogView;
use crate::app::theme::{FONT_META, Theme};
use crate::ui::icons::glyph;

pub(super) fn status_bar(view: &LogView, t: &Theme, cx: &mut Context<LogView>) -> impl IntoElement {
    let vm = view.vm.read(cx);
    let count = vm.graph.changes.len();
    let pos_label = match vm.selected {
        Some(ix) if count > 0 => format!("{} of {count}", ix + 1),
        _ if count > 0 => format!("{count} changes"),
        _ => "—".to_string(),
    };
    let repo_path = vm.repo_path.clone();
    let pr = vm.pr_info.clone();
    let workspaces = vm.graph.workspaces.clone();

    let mut left = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim));
    if let Some(current) = workspaces.iter().find(|w| w.is_current) {
        left = left.child(workspace_pill(&current.name, t));
    }
    left = left.child(repo_path);
    if let Some(pr) = pr {
        left = left.child(pr_badge(&pr, t, cx));
    }

    let right = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(pos_label));

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_3()
        .py(px(4.))
        .bg(rgb(t.status_bg))
        .border_t_1()
        .border_color(rgb(t.border))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(left)
        .child(right)
}

fn workspace_pill(name: &str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .py(px(1.))
        .rounded_sm()
        .bg(rgb(t.tag_bg))
        .text_color(rgb(t.tag_fg))
        .text_size(px(FONT_META))
        .child(crate::ui::icons::icon(glyph::COLUMNS, 10., t.tag_fg))
        .child(SharedString::from(name.to_owned()))
        .into_any_element()
}

fn pr_badge(pr: &jayjay_core::PrInfo, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    use jayjay_core::{ChecksStatus, PrState};

    let (state_label, state_fg, state_bg) = match pr.state {
        PrState::Open => ("open", t.tag_added_fg, t.tag_added_bg),
        PrState::Closed => ("closed", t.tag_removed_fg, t.tag_removed_bg),
        PrState::Merged => ("merged", t.tag_modified_fg, t.tag_modified_bg),
    };

    let (check_glyph, check_color) = match pr.checks {
        ChecksStatus::Passing => Some((glyph::CHECK, t.diff_gutter_added_fg)),
        ChecksStatus::Failing => Some((glyph::X, t.diff_gutter_removed_fg)),
        ChecksStatus::Pending => Some((glyph::DOT, t.fg_dim)),
        ChecksStatus::None => None,
    }
    .unzip();

    let url = SharedString::from(pr.url.clone());
    let label = format!("PR #{}", pr.number);

    let mut row = div()
        .id(SharedString::from(format!("pr-badge-{}", pr.number)))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .cursor_pointer()
        .on_click(cx.listener(move |_, _, _, cx| {
            cx.open_url(url.as_ref());
        }))
        .child(
            div()
                .px(px(6.))
                .py(px(1.))
                .rounded_sm()
                .bg(rgb(state_bg))
                .text_color(rgb(state_fg))
                .text_size(px(FONT_META))
                .child(SharedString::from(state_label)),
        )
        .child(div().text_color(rgb(t.fg)).child(SharedString::from(label)));

    if let (Some(g), Some(c)) = (check_glyph, check_color) {
        row = row.child(crate::ui::icons::icon(g, 11., c));
    }

    row.into_any_element()
}
