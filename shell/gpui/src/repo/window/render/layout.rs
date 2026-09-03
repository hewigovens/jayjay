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
    let selected_change = vm.selected_change();
    let change_id = selected_change.map(|c| c.change_id.id.clone());
    let show_review =
        selected_change.map(|c| c.is_working_copy).unwrap_or(false) && vm.compare.is_none();
    let mut reviewed_files = None;
    let reviewed_count = match (files.as_ref(), change_id.as_ref()) {
        (Some(fs), Some(cid)) => {
            let reviewed = fs
                .iter()
                .filter(|h| view.is_reviewed(cid, &h.path, &h.review_identity))
                .map(|h| (h.path.clone(), h.review_identity.clone()))
                .collect::<std::collections::HashSet<_>>();
            let count = reviewed.len();
            reviewed_files = Some(std::sync::Arc::new(reviewed));
            count
        }
        _ => 0,
    };
    let note_counts = vm.active_note_counts();
    let visible_indices = files.as_ref().map(|fs| {
        std::sync::Arc::new(view.visible_file_indices(
            fs,
            change_id.as_deref(),
            show_review,
            Some(&note_counts),
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
                reviewed_files,
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
