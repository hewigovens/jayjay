mod items;
mod model;

use gpui::{Context, IntoElement, ParentElement, Styled, div, px, rgb};

use super::RepoWindow;
use crate::app::theme::{FONT_META, Theme, ui_font_size};

pub(super) fn status_bar(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> impl IntoElement {
    let vm = view.vm.read(cx);
    let changes = vm.graph.changes.clone();
    let bookmarks = vm.graph.bookmarks.clone();
    let repo_path = vm.repo_path.clone();
    let pr = vm.pr_info.clone();
    let working_copy_stats = vm.working_copy_stats.clone();
    let operation = vm.current_operation_description.clone();

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px(px(12.))
        .py(px(5.))
        .bg(rgb(t.header_bg))
        .border_t_1()
        .border_color(rgb(t.border))
        .text_size(ui_font_size(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(items::status_group(
            items::leading_items(
                repo_path,
                vm.repo.clone(),
                changes.as_ref(),
                bookmarks.as_ref(),
                pr.as_ref(),
                t,
                cx,
            ),
            t,
        ))
        .child(div().flex_1())
        .child(items::status_group(
            items::trailing_items(working_copy_stats.as_ref(), &operation, t, cx),
            t,
        ))
}
