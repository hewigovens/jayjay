use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::repo::RepoWindow;
use crate::repo::revset::CompareState;
use crate::ui::icons::{glyph, icon};

pub(super) fn compare_banner(
    compare: &CompareState,
    file_count: Option<usize>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let can_reverse = compare.source_change_id.is_some() && compare.target_change_id.is_some();
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(8.))
        .bg(rgb(t.compare_bg))
        .child(compare_direction_button(can_reverse, t, cx))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(t.fg))
                .child(SharedString::from(compare.display.title.clone())),
        )
        .child(compare_label(&compare.display.from, t))
        .child(icon(glyph::ARROW_RIGHT, 10., t.fg_dim))
        .child(compare_label(&compare.display.to, t))
        .child(div().flex_1());

    if let Some(file_count) = file_count {
        row = row.child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(files_changed_label(file_count))),
        );
    }

    row.child(
        div()
            .id(SharedString::from("compare-close"))
            .flex()
            .items_center()
            .justify_center()
            .size(px(18.))
            .rounded_md()
            .cursor_pointer()
            .on_click(cx.listener(|view, _, _window, cx| {
                view.vm.update(cx, |vm, cx| vm.clear_compare(cx));
            }))
            .child(icon(glyph::X_CIRCLE, 15., t.fg_dim)),
    )
    .into_any_element()
}

fn compare_direction_button(
    can_reverse: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let button = div()
        .id(SharedString::from("compare-reverse"))
        .flex()
        .items_center()
        .justify_center()
        .size(px(20.))
        .rounded_md()
        .child(icon(glyph::ARROWS_LEFT_RIGHT, 17., t.compare_accent));

    if can_reverse {
        button
            .cursor_pointer()
            .hover(|s| s.bg(rgb(t.row_alt_bg)))
            .on_click(cx.listener(|view, _, _window, cx| {
                view.vm.update(cx, |vm, cx| vm.reverse_compare(cx));
            }))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

fn compare_label(label: &str, t: &Theme) -> AnyElement {
    div()
        .max_w(px(180.))
        .overflow_hidden()
        .font_family(crate::app::fonts::mono())
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_size(px(12.))
        .text_color(rgb(t.fg))
        .child(SharedString::from(label.to_owned()))
        .into_any_element()
}

fn files_changed_label(file_count: usize) -> String {
    if file_count == 1 {
        "1 file changed".to_string()
    } else {
        format!("{file_count} files changed")
    }
}
