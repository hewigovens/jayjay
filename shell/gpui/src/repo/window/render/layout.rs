use gpui::{AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Styled, div, px};

use crate::app::theme::Theme;
use crate::diff::{FileColumnState, file_column};
use crate::repo::window::{ActivePane, DragTarget, RepoWindow};

pub(super) fn resize_handle(
    target: DragTarget,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let debug_selector = match target {
        DragTarget::Sidebar => "sidebar-resize-handle",
        DragTarget::FileColumn => "file-column-resize-handle",
        DragTarget::OperationLog => "operation-log-resize-handle",
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
    let review_mode = view.review_mode(cx);
    let show_marks = review_mode.shows_marks();
    let vm = view.vm.read(cx);
    let files = vm.files.clone();
    // A stack shows every file, so the list marks none as the one on show.
    let selected_file_ix = vm.selected_file_ix.filter(|_| !view.shows_diff_stack());
    let loading_files = vm.loading.files;
    let change_id = vm.selected_change().map(|c| c.change_id.id.clone());
    let show_review = vm.shows_review_controls();
    let review_rollups = match (show_marks, files.as_ref(), change_id.as_ref()) {
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
    let agent_marked = std::sync::Arc::new(match (show_marks, change_id.as_ref()) {
        (true, Some(change_id)) => view.agent_marked_paths(change_id),
        _ => std::collections::HashSet::new(),
    });
    let note_counts = vm.active_note_counts();
    let conflicted = vm.conflicted_paths.clone();
    let visible_indices = files.as_ref().map(|fs| {
        std::sync::Arc::new(view.visible_file_indices(
            fs,
            show_review,
            Some(&note_counts),
            Some(&review_rollups),
            &vm.tracked_lfs_paths,
        ))
    });
    let hide_reviewed = show_marks && view.file_column.hide_reviewed;
    let notes_only = show_review && view.file_column.notes_only;
    div()
        .debug_selector(|| "file-column".to_owned())
        .w(px(width))
        .h_full()
        .child(file_column(
            FileColumnState {
                menu_anchor: view.selected_file_bounds.clone(),
                hunks: files,
                selected_ix: selected_file_ix,
                multi_selected,
                loading: loading_files,
                collapsed_dirs: &collapsed,
                scroll,
                tree_scroll,
                change_id,
                review_rollups,
                agent_marked,
                reviewed_count,
                show_review: show_marks,
                review_mode,
                hide_reviewed,
                note_counts,
                conflicted,
                notes_only,
                file_filter: view.file_column.filter.as_ref(),
                file_filter_focus: view.file_filter_focus.clone(),
                visible_indices,
                column_width: width,
                tree_cache,
                pane_active: view.active_pane() == ActivePane::FileColumn,
                shows_all_files: view.shows_all_file_diffs(cx),
                focused: view.focused_control(),
            },
            cx,
        ))
        .into_any_element()
}
