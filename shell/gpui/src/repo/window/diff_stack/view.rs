use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

use super::plan::{StackBlock, StackBody, StackMessage, StackMessageAction, StackPlan};
use crate::app::theme::Theme;
use crate::diff::{
    StackedFileHeader, StackedRows, image_diff_view, placeholder_row, stacked_file_header,
};
use crate::repo::window::{RepoWindow, hunk_supports_file_editor};
use crate::ui::loading::loading_status;
use crate::ui::scrollbar::vertical_scrollbar;

pub(crate) fn diff_stack(
    view: &RepoWindow,
    plan: &StackPlan,
    detail_width: f32,
    t: &Theme,
    window: &mut Window,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let scroll = view.diff.stack.scroll.clone();
    let top = plan.layout.top(plan.first);
    let bottom = plan.layout.top(plan.first + plan.blocks.len());
    let (shows_review, file_reviews, working_copy, has_conflict, query, selected, stats, copied) = {
        let vm = view.vm.read(cx);
        let hunks = plan.blocks.iter().map(|block| &plan.files[block.hunk_ix]);
        let file_reviews = vm
            .selected_change()
            .filter(|_| view.shows_review_marks(cx))
            .map(|change| view.review_rollups_with_vm(&change.change_id.id, hunks, vm))
            .unwrap_or_default();
        (
            vm.shows_review_controls(),
            file_reviews,
            vm.selected_change()
                .is_some_and(|change| change.is_working_copy),
            vm.conflicted_paths.clone(),
            view.find
                .query
                .as_ref()
                .map(|query| query.text().to_owned())
                .filter(|query| !query.is_empty()),
            vm.selected_file_ix,
            vm.stats.per_file.clone(),
            view.feedback.recently_copied.as_ref().map(|s| s.as_ref()) == Some("path"),
        )
    };
    let frame = StackPlan::frame_height(t);

    let mut list = div()
        .id("diff-stack")
        .debug_selector(|| "diff-stack".to_owned())
        .flex()
        .flex_col()
        .size_full()
        .overflow_y_scroll()
        .track_scroll(&scroll)
        .child(div().flex_none().h(px(top)));
    let header_of = |built: &StackBlock, cx: &mut Context<RepoWindow>| {
        let hunk = &plan.files[built.hunk_ix];
        let can_edit_file = matches!(
            built.body,
            StackBody::Rows {
                supports_file_editor: true,
                ..
            } | StackBody::Message(StackMessage {
                action: Some(StackMessageAction::ShowDiff {
                    supports_file_editor: true
                }),
                ..
            })
        ) && working_copy
            && hunk_supports_file_editor(hunk)
            && !has_conflict.contains(&hunk.path);
        stacked_file_header(
            StackedFileHeader {
                hunk,
                line_stats: stats.get(&hunk.path),
                just_copied: copied && selected == Some(built.hunk_ix),
                can_edit_file,
                file_review: file_reviews.get(&hunk.path).copied(),
                collapsed: matches!(built.body, StackBody::Collapsed),
                detail_width,
            },
            t,
            cx,
        )
    };
    // The file under the top edge keeps its header there until the next file's header pushes it out.
    let scroll_y = -f32::from(scroll.offset().y);
    let pinned = plan
        .layout
        .locate(scroll_y)
        .filter(|(_, offset)| *offset > 0.)
        .and_then(|(block, offset)| {
            let built = plan.blocks.get(block.checked_sub(plan.first)?)?;
            let hunk_ix = built.hunk_ix;
            let below = plan.layout.top(block + 1) - plan.layout.top(block) - offset;
            Some(
                div()
                    .id(("stack-pinned", hunk_ix))
                    .debug_selector(|| "stack-pinned-header".to_owned())
                    .absolute()
                    .top(px((below - frame).min(0.)))
                    .left_0()
                    .right_0()
                    .occlude()
                    .capture_any_mouse_down(cx.listener(move |view, _, _, cx| {
                        view.activate_stacked_file(hunk_ix, cx);
                    }))
                    .child(header_of(built, cx)),
            )
        });

    for built in &plan.blocks {
        let StackBlock {
            hunk_ix,
            body_height,
            body,
        } = built;
        let hunk_ix = *hunk_ix;
        let hunk = &plan.files[hunk_ix];
        let active = selected == Some(hunk_ix);
        let header = header_of(built, cx);
        let content = match body {
            StackBody::Rows {
                diff,
                view_mode,
                notes,
                wrap,
                ..
            } => {
                let review = view.review_display_state(Some(hunk), Some(diff), wrap, cx);
                StackedRows {
                    file_diff: diff,
                    view_mode: *view_mode,
                    notes,
                    wrap_cache: wrap,
                    unified_bounds: &view.diff.unified_bounds,
                    sbs_old_bounds: &view.diff.sbs_old_bounds,
                    sbs_new_bounds: &view.diff.sbs_new_bounds,
                }
                .render(
                    active,
                    query.clone().filter(|_| active),
                    shows_review,
                    review,
                    t,
                    cx,
                )
            }
            StackBody::Image => image_diff_view(hunk, t, window, cx),
            StackBody::Loading => div()
                .debug_selector(|| "stack-loading".to_owned())
                .size_full()
                .child(loading_status(
                    jayjay_i18n::diff::loading_diff_ellipsis(),
                    t,
                ))
                .into_any_element(),
            StackBody::Message(message) => {
                let action = message.action.map(|action| {
                    let (selector, label) = match action {
                        StackMessageAction::ShowDiff { .. } => {
                            ("stack-show-diff", jayjay_i18n::diff::show_diff())
                        }
                        StackMessageAction::Retry => ("stack-retry", jayjay_i18n::repo::retry()),
                    };
                    div()
                        .debug_selector(move || selector.to_owned())
                        .cursor_pointer()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(rgb(t.selected_accent))
                        .hover(|style| style.underline())
                        .child(label)
                        // Acts on press: the block's capture handler selects the file first, and that reload can replace this row before the release.
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |view, _, _, cx| match action {
                                StackMessageAction::ShowDiff { .. } => {
                                    view.show_stacked_file_alone(hunk_ix, cx)
                                }
                                StackMessageAction::Retry => view.retry_stacked_diffs(cx),
                            }),
                        )
                        .into_any_element()
                });
                placeholder_row(
                    &hunk.path,
                    message.glyph,
                    message.title.clone(),
                    message.detail.clone(),
                    action,
                    t,
                )
            }
            StackBody::Collapsed => div().into_any_element(),
        };
        let path = hunk.path.clone();
        list = list.child(
            div()
                .id(("stack-file", hunk_ix))
                .debug_selector(move || format!("stack-file-{path}"))
                .flex()
                .flex_col()
                .flex_none()
                .h(px(frame + body_height))
                .overflow_hidden()
                .border_b_1()
                .border_color(rgb(t.border))
                .capture_any_mouse_down(cx.listener(move |view, _, _, cx| {
                    view.activate_stacked_file(hunk_ix, cx);
                }))
                .child(header)
                .child(div().flex_none().h(px(*body_height)).child(content)),
        );
    }
    list = list.child(div().flex_none().h(px(plan.layout.total() - bottom)));

    div()
        .relative()
        .flex()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .overflow_hidden()
        .bg(rgb(t.detail_bg))
        .child(list)
        .children(pinned)
        .child(vertical_scrollbar(scroll, t, cx))
        .into_any_element()
}
