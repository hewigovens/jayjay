mod flat;
mod header;
mod row;
mod tree;

use std::sync::Arc;

use gpui::{
    AnyElement, Context, IntoElement, ParentElement, Styled, UniformListScrollHandle, div, rgb,
};
use jayjay_core::file_tree::build_file_tree;
use jayjay_core::{DiffHunk, FileTreeEntry};

use crate::app::config;
use crate::app::theme::theme;
use crate::repo::window::RepoWindow;

use flat::flat_body;
use header::file_column_header;
use tree::{is_entry_visible, tree_body};

/// Inputs for the file column body.
pub struct FileColumnState<'a> {
    pub hunks: Option<&'a [DiffHunk]>,
    pub selected_ix: Option<usize>,
    pub loading: bool,
    pub collapsed_dirs: &'a std::collections::HashSet<String>,
    pub scroll: UniformListScrollHandle,
    pub change_id: Option<String>,
    pub reviewed_count: usize,
    /// Review checkboxes only render for the working copy.
    pub show_review: bool,
    /// Container width in px — used to size middle-truncation char budgets.
    pub column_width: f32,
}

pub fn file_column(state: FileColumnState<'_>, cx: &mut Context<RepoWindow>) -> AnyElement {
    let FileColumnState {
        hunks,
        selected_ix,
        loading,
        collapsed_dirs,
        scroll,
        change_id,
        reviewed_count,
        show_review,
        column_width,
    } = state;
    let t = theme(cx).clone();
    let cfg = config::current(cx);
    let tree_mode = cfg.diff.tree_file_list;

    let hunks = match hunks {
        Some(h) if !h.is_empty() => h,
        _ => {
            let label = if loading {
                "Loading files…"
            } else {
                "No files"
            };
            return div()
                .flex()
                .flex_col()
                .size_full()
                .bg(rgb(t.sidebar_bg))
                .child(file_column_header(0, 0, loading, show_review, &t))
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(t.fg_dim))
                        .child(label),
                )
                .into_any_element();
        }
    };

    let hunks: Arc<Vec<DiffHunk>> = Arc::new(hunks.to_vec());
    let count = hunks.len();

    let body = if tree_mode {
        let paths: Vec<String> = hunks.iter().map(|h| h.path.clone()).collect();
        let full_tree = build_file_tree(&paths);
        let visible: Vec<FileTreeEntry> = full_tree
            .into_iter()
            .filter(|e| is_entry_visible(e, collapsed_dirs))
            .collect();
        let tree: Arc<Vec<FileTreeEntry>> = Arc::new(visible);
        tree_body(
            hunks.clone(),
            tree,
            selected_ix,
            collapsed_dirs.clone(),
            t.clone(),
            scroll,
            change_id.clone(),
            show_review,
            cx,
        )
    } else {
        flat_body(
            hunks.clone(),
            selected_ix,
            t.clone(),
            scroll,
            change_id.clone(),
            show_review,
            column_width,
            cx,
        )
    };

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(rgb(t.sidebar_bg))
        .child(file_column_header(
            reviewed_count,
            count,
            loading,
            show_review,
            &t,
        ))
        .child(body)
        .into_any_element()
}
