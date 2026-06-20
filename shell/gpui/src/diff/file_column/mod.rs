mod flat;
mod header;
mod row;
mod tree;
mod tree_cache;

use std::sync::Arc;

use gpui::{
    AnyElement, Context, IntoElement, ParentElement, Styled, UniformListScrollHandle, div, rgb,
};
use jayjay_core::DiffHunk;

use crate::app::config;
use crate::app::theme::theme;
use crate::repo::window::{FileTreeCacheSlot, RepoWindow};

use flat::flat_body;
use header::file_column_header;
use tree::tree_body;

pub(crate) use tree_cache::FileTreeCache;

/// Inputs for the file column body.
pub struct FileColumnState<'a> {
    /// Shared `Arc` with `vm.files`, keeping hunk data out of the per-frame copy path.
    pub hunks: Option<Arc<Vec<DiffHunk>>>,
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
    /// Per-window cache so tree mode reuses the built tree across render frames.
    pub(crate) tree_cache: FileTreeCacheSlot,
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
        tree_cache,
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
                .child(file_column_header(
                    0,
                    0,
                    loading,
                    show_review,
                    tree_mode,
                    &t,
                ))
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

    let count = hunks.len();

    let body = if tree_mode {
        let tree = tree_cache.borrow_mut().visible(&hunks, collapsed_dirs);
        tree_body(
            hunks.clone(),
            tree,
            selected_ix,
            collapsed_dirs.clone(),
            t.clone(),
            scroll,
            change_id.clone(),
            show_review,
            column_width,
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
            tree_mode,
            &t,
        ))
        .child(body)
        .into_any_element()
}
