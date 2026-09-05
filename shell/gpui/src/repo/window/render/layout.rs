use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div, px};

use crate::app::theme::Theme;
use crate::diff::{FileColumnState, file_column};
use crate::repo::window::{DragTarget, RepoWindow};

pub(super) fn resize_handle(
    target: DragTarget,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let debug_selector = match target {
        DragTarget::Sidebar => "sidebar-resize-handle",
        DragTarget::FileColumn => "file-column-resize-handle",
        DragTarget::Description => "description-resize-handle",
    };
    crate::ui::resize_handle::resize_handle(
        debug_selector,
        t,
        move |view, x, viewport_width, cx| view.start_drag(target, x, viewport_width, cx),
        cx,
    )
}

pub(super) fn file_column_wrapper(
    view: &RepoWindow,
    width: f32,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let collapsed = view.collapsed_dirs.clone();
    let scroll = view.scrolls.files.clone();
    let tree_scroll = view.scrolls.tree_files.clone();
    let tree_cache = view.file_tree_cache.clone();
    let multi_selected = view.multi_selected_hunk_indices();
    let vm = view.vm.read(cx);
    let files = vm.files.clone();
    let selected_file_ix = vm.selected_file_ix;
    let loading_files = vm.loading.files;
    let change_id = vm.selected_change().map(|c| c.change_id.id.clone());
    let show_review = vm.shows_review_controls();
    let review_rollups = match (show_review, files.as_ref(), change_id.as_ref()) {
        (true, Some(files), Some(change_id)) => {
            view.review_rollups_with_vm(change_id, files.iter(), vm)
        }
        _ => std::collections::HashMap::new(),
    };
    let reviewed_count = review_rollups
        .values()
        .filter(|rollup| **rollup == jayjay_review::ReviewFileRollup::Reviewed)
        .count();
    let review_rollups = std::sync::Arc::new(review_rollups);
    let note_counts = vm.active_note_counts();
    let visible_indices = files.as_ref().map(|fs| {
        std::sync::Arc::new(view.visible_file_indices(
            fs,
            show_review,
            Some(&note_counts),
            Some(&review_rollups),
        ))
    });
    let hide_reviewed = show_review && view.file_column.hide_reviewed;
    let notes_only = show_review && view.file_column.notes_only;
    div()
        .w(px(width))
        .h_full()
        .child(file_column(
            FileColumnState {
                hunks: files,
                selected_ix: selected_file_ix,
                multi_selected,
                loading: loading_files,
                collapsed_dirs: &collapsed,
                scroll,
                tree_scroll,
                change_id,
                review_rollups,
                reviewed_count,
                show_review,
                hide_reviewed,
                note_counts,
                notes_only,
                file_filter: view.file_column.filter.as_ref(),
                file_filter_focus: view.file_filter_focus.clone(),
                visible_indices,
                column_width: width,
                tree_cache,
            },
            cx,
        ))
        .into_any_element()
}
