use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px, rgb,
};
use jayjay_core::{DiffHunk, FileTreeEntry};

use super::row::{
    FileRowHandlers, FileRowState, file_name_opacity, file_row_height, file_text_content,
    file_text_inset, file_text_limits, finish_file_row, review_checkbox, row_bg, row_separator,
};
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};

const TREE_DIR_ROW_HEIGHT: f32 = 28.;
const TREE_ROW_HORIZONTAL_MARGIN: f32 = 4.;
const TREE_ROW_VERTICAL_MARGIN: f32 = 0.;
const TREE_ROW_GAP: f32 = 0.;

pub(super) fn is_entry_visible(
    entry: &FileTreeEntry,
    collapsed: &std::collections::HashSet<String>,
) -> bool {
    for prefix in collapsed {
        let with_slash = format!("{prefix}/");
        if entry.path.starts_with(&with_slash) {
            return false;
        }
    }
    true
}

pub(super) struct TreeBodyState {
    pub(super) hunks: Arc<Vec<DiffHunk>>,
    pub(super) visible_indices: Arc<Vec<usize>>,
    pub(super) tree: Arc<Vec<FileTreeEntry>>,
    pub(super) selected_ix: Option<usize>,
    pub(super) multi_selected: Arc<HashSet<usize>>,
    pub(super) collapsed: std::collections::HashSet<String>,
    pub(super) theme: Theme,
    pub(super) scroll: ScrollHandle,
    pub(super) change_id: Option<String>,
    pub(super) reviewed_files: Option<Arc<HashSet<(String, String)>>>,
    pub(super) show_review: bool,
    pub(super) note_counts: Arc<std::collections::HashMap<String, usize>>,
    pub(super) column_width: f32,
}

pub(super) fn tree_body(state: TreeBodyState, cx: &mut Context<RepoWindow>) -> AnyElement {
    let TreeBodyState {
        hunks,
        visible_indices,
        tree,
        selected_ix,
        multi_selected,
        collapsed,
        theme,
        scroll,
        change_id,
        reviewed_files,
        show_review,
        note_counts,
        column_width,
    } = state;
    let collapsed = Arc::new(collapsed);
    let mut list = div()
        .id("files-tree")
        .debug_selector(|| "files-tree".to_owned())
        .flex()
        .flex_col()
        .gap(px(TREE_ROW_GAP))
        .h_full()
        .px(px(TREE_ROW_HORIZONTAL_MARGIN))
        .py(px(TREE_ROW_VERTICAL_MARGIN))
        .overflow_y_scroll()
        .scrollbar_width(px(0.))
        .track_scroll(&scroll);
    let row_width = (column_width - TREE_ROW_HORIZONTAL_MARGIN * 2.).max(0.);

    for (ix, entry) in tree.iter().enumerate() {
        let row = if let Some(hunk_ix) = entry.hunk_index {
            let visible_hunk_ix = hunk_ix as usize;
            match visible_indices.get(visible_hunk_ix).copied() {
                Some(hunk_ix) => {
                    let is_selected =
                        selected_ix == Some(hunk_ix) || multi_selected.contains(&hunk_ix);
                    if let Some(hunk) = hunks.get(hunk_ix) {
                        let path = hunk.path.clone();
                        let identity = hunk.review_identity.clone();
                        let show_review = show_review && !identity.is_empty();
                        let path_for_review = path.clone();
                        let identity_for_review = identity.clone();
                        let change_for_review = change_id.clone();
                        let reviewed = reviewed_files
                            .as_ref()
                            .is_some_and(|files| files.contains(&(path.clone(), identity.clone())));
                        let note_count = note_counts.get(&path).copied().unwrap_or(0);
                        tree_file_row(
                            entry,
                            FileRowState {
                                hunk,
                                is_selected,
                                reviewed,
                                show_review,
                                note_count,
                                ix,
                                theme: &theme,
                            },
                            row_width,
                            FileRowHandlers {
                                on_click: cx.listener(
                                    move |view, event: &ClickEvent, _window, cx| {
                                        view.handle_file_row_click(hunk_ix, event.modifiers(), cx);
                                    },
                                ),
                                on_right_click: cx.listener(
                                    move |view, ev: &MouseDownEvent, _w, cx| {
                                        view.open_file_context_menu(&path, ev.position, cx);
                                    },
                                ),
                                on_review_click: cx.listener(
                                    move |view, _event: &ClickEvent, _w, cx| {
                                        if let Some(cid) = change_for_review.clone() {
                                            view.toggle_reviewed(
                                                cid,
                                                path_for_review.clone(),
                                                identity_for_review.clone(),
                                                cx,
                                            );
                                        }
                                    },
                                ),
                            },
                        )
                    } else {
                        div().into_any_element()
                    }
                }
                None => div().into_any_element(),
            }
        } else {
            let is_collapsed = collapsed.contains(&entry.path);
            let dir_path = entry.path.clone();
            tree_dir_row(
                entry,
                ix,
                is_collapsed,
                &theme,
                cx.listener(move |view, _event, _window, cx| {
                    view.toggle_dir(dir_path.clone(), cx);
                }),
            )
        };
        list = list.child(row);
    }

    list.into_any_element()
}

fn tree_file_row<F, FR, FRev>(
    entry: &FileTreeEntry,
    state: FileRowState<'_>,
    column_width: f32,
    handlers: FileRowHandlers<F, FR, FRev>,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    FR: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let FileRowState {
        hunk,
        is_selected,
        reviewed,
        show_review,
        note_count,
        ix,
        theme,
    } = state;
    let FileRowHandlers {
        on_click,
        on_right_click,
        on_review_click,
    } = handlers;
    let bg_row = row_bg(is_selected, theme);
    let indent = (entry.depth as f32) * 14.0;
    let name_opacity = file_name_opacity(show_review, reviewed);
    let fixed_chrome = if show_review { 80.0 } else { 56.0 };
    let text_px = (column_width - fixed_chrome - indent).max(80.0);
    let (basename_chars, path_chars) = file_text_limits(text_px, theme);
    let name = super::flat::middle_elide(&entry.name, basename_chars);
    let path_display = super::flat::middle_elide(&hunk.path, path_chars);
    let content = file_text_content(
        SharedString::from(name),
        SharedString::from(path_display),
        name_opacity,
        theme,
    );
    let mut row = div()
        .id(("tree-file", ix))
        .debug_selector(move || format!("tree-file-{ix}"))
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(8.))
        .pl(px(6. + indent))
        .pr(px(6.))
        .h(px(file_row_height(theme)))
        .rounded_md()
        .bg(rgb(bg_row))
        .relative()
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click)
        .child(row_separator(
            6. + indent + file_text_inset(show_review),
            theme,
        ));
    if show_review {
        row = row.child(review_checkbox(
            ("review-tree", ix),
            reviewed,
            theme,
            on_review_click,
        ));
    }
    finish_file_row(row, hunk, content, note_count, theme)
}

fn tree_dir_row<F>(
    entry: &FileTreeEntry,
    ix: usize,
    is_collapsed: bool,
    t: &Theme,
    on_click: F,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let indent = (entry.depth as f32) * 14.0;
    let chevron_glyph = if is_collapsed {
        glyph::CARET_RIGHT
    } else {
        glyph::CARET_DOWN
    };
    div()
        .id(("tree-dir", ix))
        .debug_selector(move || format!("tree-dir-{ix}"))
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(4.))
        .pl(px(6. + indent))
        .pr(px(6.))
        .h(px(TREE_DIR_ROW_HEIGHT + t.scaled_font_size(12.) - 12.))
        .relative()
        .cursor_pointer()
        .on_click(on_click)
        .child(row_separator(36. + indent, t))
        .child(icons::icon(chevron_glyph, 10., t.fg_faint))
        .child(icons::icon(glyph::FOLDER_SIMPLE, 12., t.fg_dim))
        .child(super::file_name_container(
            div()
                .flex_1()
                .min_w_0()
                .font_family(fonts::mono())
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(entry.name.clone())),
        ))
        .into_any_element()
}
