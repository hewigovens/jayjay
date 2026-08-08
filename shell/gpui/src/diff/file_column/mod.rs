mod flat;
mod header;
mod row;
mod tree;
mod tree_cache;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{
    AnyElement, Context, IntoElement, ParentElement, ScrollHandle, Styled, UniformListScrollHandle,
    div, rgb,
};
use jayjay_core::DiffHunk;

use crate::app::config;
use crate::app::theme::theme;
use crate::repo::window::{FileTreeCacheSlot, RepoWindow};

fn file_name_container(name: impl IntoElement) -> impl IntoElement {
    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_row()
        .items_center()
        .min_w_0()
        .child(name)
}

use flat::{FlatBodyState, flat_body};
use header::{FileHeaderFilters, file_column_header};
use tree::{TreeBodyState, tree_body};

pub(crate) use flat::middle_elide;

pub(crate) use tree_cache::FileTreeCache;

pub struct FileColumnState<'a> {
    pub(crate) hunks: Option<Arc<Vec<DiffHunk>>>,
    pub(crate) selected_ix: Option<usize>,
    /// Hunk indices in the multi-selection; highlighted like the primary row and targeted by batch context-menu actions.
    pub(crate) multi_selected: Arc<HashSet<usize>>,
    pub(crate) loading: bool,
    pub(crate) collapsed_dirs: &'a std::collections::HashSet<String>,
    pub(crate) scroll: UniformListScrollHandle,
    pub(crate) tree_scroll: ScrollHandle,
    pub(crate) change_id: Option<String>,
    pub(crate) reviewed_files: Option<Arc<HashSet<(String, String)>>>,
    pub(crate) reviewed_count: usize,
    pub(crate) show_review: bool,
    pub(crate) hide_reviewed: bool,
    /// Empty when the notes session isn't active; drives both the per-row badge and the header's noted-files filter toggle.
    pub(crate) note_counts: Arc<HashMap<String, usize>>,
    pub(crate) notes_only: bool,
    pub(crate) visible_indices: Option<Arc<Vec<usize>>>,
    pub(crate) column_width: f32,
    /// Per-window cache so tree mode reuses the built tree across render frames.
    pub(crate) tree_cache: FileTreeCacheSlot,
}

pub fn file_column(state: FileColumnState<'_>, cx: &mut Context<RepoWindow>) -> AnyElement {
    let FileColumnState {
        hunks,
        selected_ix,
        multi_selected,
        loading,
        collapsed_dirs,
        scroll,
        tree_scroll,
        change_id,
        reviewed_files,
        reviewed_count,
        show_review,
        hide_reviewed,
        note_counts,
        notes_only,
        visible_indices,
        column_width,
        tree_cache,
    } = state;
    let t = theme(cx).clone();
    let cfg = config::current(cx);
    let tree_mode = cfg.diff.tree_file_list;
    let active_note_count: usize = note_counts.values().sum();

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
                .bg(rgb(t.detail_bg))
                .child(file_column_header(
                    0,
                    0,
                    loading,
                    &FileHeaderFilters {
                        show_review,
                        hide_reviewed,
                        active_note_count: 0,
                        notes_only: false,
                    },
                    tree_mode,
                    cx,
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
    let visible_indices = visible_indices.unwrap_or_else(|| Arc::new((0..count).collect()));

    let body = if tree_mode {
        let tree = tree_cache
            .borrow_mut()
            .visible(&hunks, &visible_indices, collapsed_dirs);
        tree_body(
            TreeBodyState {
                hunks: hunks.clone(),
                visible_indices: visible_indices.clone(),
                tree,
                selected_ix,
                multi_selected,
                collapsed: collapsed_dirs.clone(),
                theme: t.clone(),
                scroll: tree_scroll,
                change_id: change_id.clone(),
                reviewed_files: reviewed_files.clone(),
                show_review,
                note_counts: note_counts.clone(),
                column_width,
            },
            cx,
        )
    } else {
        flat_body(
            FlatBodyState {
                hunks: hunks.clone(),
                visible_indices: visible_indices.clone(),
                selected_ix,
                multi_selected,
                theme: t.clone(),
                scroll,
                change_id: change_id.clone(),
                show_review,
                note_counts: note_counts.clone(),
                column_width,
            },
            cx,
        )
    };

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(rgb(t.detail_bg))
        .child(file_column_header(
            reviewed_count,
            count,
            loading,
            &FileHeaderFilters {
                show_review,
                hide_reviewed,
                active_note_count,
                notes_only,
            },
            tree_mode,
            cx,
            &t,
        ))
        .child(body)
        .into_any_element()
}
