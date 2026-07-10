use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px, rgb,
};
use jayjay_core::{DiffHunk, FileTreeEntry};

use super::row::{file_name_opacity, file_text_content, finish_file_row, review_checkbox, row_bg};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};

const TREE_FILE_ROW_HEIGHT: f32 = 46.;
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

#[allow(clippy::too_many_arguments)]
pub(super) fn tree_body(
    hunks: Arc<Vec<DiffHunk>>,
    visible_indices: Arc<Vec<usize>>,
    tree: Arc<Vec<FileTreeEntry>>,
    selected_ix: Option<usize>,
    collapsed: std::collections::HashSet<String>,
    t: Theme,
    scroll: ScrollHandle,
    change_id: Option<String>,
    reviewed_files: Option<Arc<HashSet<(String, String)>>>,
    show_review: bool,
    note_counts: Arc<std::collections::HashMap<String, usize>>,
    column_width: f32,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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
                    let is_selected = selected_ix == Some(hunk_ix);
                    if let Some(hunk) = hunks.get(hunk_ix) {
                        let path = hunk.path.clone();
                        let identity = hunk.review_identity.clone();
                        let path_for_review = path.clone();
                        let identity_for_review = identity.clone();
                        let change_for_review = change_id.clone();
                        let reviewed = reviewed_files
                            .as_ref()
                            .is_some_and(|files| files.contains(&(path.clone(), identity.clone())));
                        let note_count = note_counts.get(&path).copied().unwrap_or(0);
                        tree_file_row(
                            entry,
                            hunk,
                            is_selected,
                            reviewed,
                            show_review,
                            note_count,
                            ix,
                            row_width,
                            &t,
                            cx.listener(move |view, _event, _window, cx| {
                                view.select_file(hunk_ix, cx);
                            }),
                            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                                let items = RepoWindow::build_file_menu(&path, cx);
                                view.open_context_menu(ev.position, items, cx);
                            }),
                            cx.listener(move |view, _event: &ClickEvent, _w, cx| {
                                if let Some(cid) = change_for_review.clone() {
                                    view.toggle_reviewed(
                                        cid,
                                        path_for_review.clone(),
                                        identity_for_review.clone(),
                                        cx,
                                    );
                                }
                            }),
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
                &t,
                cx.listener(move |view, _event, _window, cx| {
                    view.toggle_dir(dir_path.clone(), cx);
                }),
            )
        };
        list = list.child(row);
    }

    list.into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn tree_file_row<F, FR, FRev>(
    entry: &FileTreeEntry,
    hunk: &DiffHunk,
    is_selected: bool,
    reviewed: bool,
    show_review: bool,
    note_count: usize,
    ix: usize,
    column_width: f32,
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
    let bg_row = row_bg(is_selected, t);
    let indent = (entry.depth as f32) * 14.0;
    let name_opacity = file_name_opacity(show_review, reviewed);
    let fixed_chrome = if show_review { 80.0 } else { 56.0 };
    let text_px = (column_width - fixed_chrome - indent).max(80.0);
    let basename_chars = ((text_px / 7.2) as usize).max(8);
    let path_chars = ((text_px / 6.0) as usize).max(10);
    let name = super::flat::middle_elide(&entry.name, basename_chars);
    let path_display = super::flat::middle_elide(&hunk.path, path_chars);
    let content = file_text_content(
        SharedString::from(name),
        SharedString::from(path_display),
        name_opacity,
        t,
    );
    let mut row = div()
        .id(("tree-file", ix))
        .debug_selector(move || format!("tree-file-{ix}"))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(8.))
        .pl(px(6. + indent))
        .pr(px(6.))
        .h(px(TREE_FILE_ROW_HEIGHT))
        .rounded_md()
        .border_b_1()
        .border_color(rgb(t.row_border))
        .bg(rgb(bg_row))
        .relative()
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click);
    if show_review {
        row = row.child(review_checkbox(
            ("review-tree", ix),
            reviewed,
            t,
            on_review_click,
        ));
    }
    finish_file_row(row, hunk, content, note_count, t)
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
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(4.))
        .pl(px(6. + indent))
        .pr(px(6.))
        .h(px(TREE_DIR_ROW_HEIGHT))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .relative()
        .cursor_pointer()
        .on_click(on_click)
        .child(icons::icon(chevron_glyph, 10., t.fg_faint))
        .child(icons::icon(glyph::FOLDER_SIMPLE, 12., t.fg_dim))
        .child(super::file_name_container(
            div()
                .flex_1()
                .min_w_0()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(entry.name.clone())),
        ))
        .into_any_element()
}
