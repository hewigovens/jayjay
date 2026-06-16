use gpui::{
    AnyElement, Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::diff::{FileColumnState, file_column};
use crate::repo::window::{DragTarget, RepoWindow};

pub(super) fn resize_handle(
    target: DragTarget,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .flex_none()
        .w(px(5.))
        .h_full()
        .cursor(CursorStyle::ResizeLeftRight)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.start_drag(target, f32::from(ev.position.x), cx);
            }),
        )
        .child(div().w(px(1.)).h_full().ml(px(2.)).bg(rgb(t.border)))
        .into_any_element()
}

pub(super) fn file_column_wrapper(
    view: &RepoWindow,
    width: f32,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let collapsed = view.collapsed_dirs.clone();
    let scroll = view.scrolls.files.clone();
    let tree_cache = view.file_tree_cache.clone();
    let vm = view.vm.read(cx);
    let files = vm.files.clone();
    let selected_file_ix = vm.selected_file_ix;
    let loading_files = vm.loading.files;
    let selected_change = vm.selected_change();
    let change_id = selected_change.map(|c| c.change_id.clone());
    let show_review =
        selected_change.map(|c| c.is_working_copy).unwrap_or(false) && vm.compare.is_none();
    let reviewed_count = match (files.as_ref(), change_id.as_ref()) {
        (Some(fs), Some(cid)) => fs
            .iter()
            .filter(|h| view.is_reviewed(cid, &h.path, &h.review_identity))
            .count(),
        _ => 0,
    };
    div()
        .w(px(width))
        .h_full()
        .child(file_column(
            FileColumnState {
                hunks: files,
                selected_ix: selected_file_ix,
                loading: loading_files,
                collapsed_dirs: &collapsed,
                scroll,
                change_id,
                reviewed_count,
                show_review,
                column_width: width,
                tree_cache,
            },
            cx,
        ))
        .into_any_element()
}
