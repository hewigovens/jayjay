use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, px, rgb, uniform_list,
};
use jayjay_core::DiffHunk;

use super::row::{review_checkbox, row_bg, status_dot};
use crate::app::fonts;
use crate::ui::primitives::no_scrollbar_gutter;
use crate::app::theme::Theme;
use crate::log::LogView;

#[allow(clippy::too_many_arguments)]
pub(super) fn flat_body(
    hunks: Arc<Vec<DiffHunk>>,
    selected_ix: Option<usize>,
    t: Theme,
    scroll: UniformListScrollHandle,
    change_id: Option<String>,
    show_review: bool,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let count = hunks.len();
    let list = uniform_list(
        "files-flat",
        count,
        cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
            let t = t.clone();
            let change_id = change_id.clone();
            range
                .map(|ix| {
                    let is_selected = selected_ix == Some(ix);
                    let path = hunks[ix].path.clone();
                    let path_for_review = path.clone();
                    let change_for_review = change_id.clone();
                    let reviewed = match change_id.as_ref() {
                        Some(cid) => this.is_reviewed(cid, &path),
                        None => false,
                    };
                    flat_file_row(
                        &hunks[ix],
                        is_selected,
                        reviewed,
                        show_review,
                        ix,
                        &t,
                        cx.listener(move |view, _event, _window, cx| {
                            view.select_file(ix, cx);
                        }),
                        cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                            let items = LogView::build_file_menu(&path);
                            view.open_context_menu(ev.position, items, cx);
                        }),
                        cx.listener(move |view, _event: &ClickEvent, _w, cx| {
                            if let Some(cid) = change_for_review.clone() {
                                view.toggle_reviewed(cid, path_for_review.clone(), cx);
                            }
                        }),
                    )
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);
    no_scrollbar_gutter(list)
        .h_full()
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn flat_file_row<F, FR, FRev>(
    hunk: &DiffHunk,
    is_selected: bool,
    reviewed: bool,
    show_review: bool,
    ix: usize,
    t: &Theme,
    on_click: F,
    on_right_click: FR,
    on_review_click: FRev,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    FR: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let bg_row = row_bg(is_selected, ix, t);

    let basename = hunk
        .path
        .rsplit('/')
        .next()
        .unwrap_or(&hunk.path)
        .to_owned();

    let name_color = if reviewed { t.fg_faint } else { t.fg };
    let content = div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .flex_1()
        .min_w_0()
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(name_color))
                .child(SharedString::from(basename)),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(10.))
                .text_color(rgb(t.fg_faint))
                .truncate()
                .child(SharedString::from(hunk.path.clone())),
        );

    let mut row = div()
        .id(("file", ix))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(44.))
        .gap(px(8.))
        .px(px(10.))
        .bg(rgb(bg_row))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click);
    if show_review {
        row = row.child(review_checkbox(
            ("review-flat", ix),
            reviewed,
            t,
            on_review_click,
        ));
    }
    row.child(status_dot(hunk))
        .child(content)
        .into_any_element()
}
