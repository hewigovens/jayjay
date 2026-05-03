use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, px, rgb, uniform_list,
};
use jayjay_core::{DiffHunk, FileTreeEntry};

use super::row::{review_checkbox, row_bg, status_dot};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::log::LogView;
use crate::ui::icons::{self, glyph};

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
    tree: Arc<Vec<FileTreeEntry>>,
    selected_ix: Option<usize>,
    collapsed: std::collections::HashSet<String>,
    t: Theme,
    scroll: UniformListScrollHandle,
    change_id: Option<String>,
    show_review: bool,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let count = tree.len();
    let collapsed = Arc::new(collapsed);
    let list = uniform_list(
        "files-tree",
        count,
        cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
            let t = t.clone();
            let hunks = hunks.clone();
            let tree = tree.clone();
            let collapsed = collapsed.clone();
            let change_id = change_id.clone();
            range
                .map(|ix| {
                    let entry = &tree[ix];
                    if let Some(hunk_ix) = entry.hunk_index {
                        let hunk_ix = hunk_ix as usize;
                        let is_selected = selected_ix == Some(hunk_ix);
                        if let Some(hunk) = hunks.get(hunk_ix) {
                            let path = hunk.path.clone();
                            let path_for_review = path.clone();
                            let change_for_review = change_id.clone();
                            let reviewed = match change_id.as_ref() {
                                Some(cid) => this.is_reviewed(cid, &path),
                                None => false,
                            };
                            return tree_file_row(
                                entry,
                                hunk,
                                is_selected,
                                reviewed,
                                show_review,
                                ix,
                                &t,
                                cx.listener(move |view, _event, _window, cx| {
                                    view.select_file(hunk_ix, cx);
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
                            );
                        }
                    }
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
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);
    crate::ui::primitives::no_scrollbar_gutter(list)
        .h_full()
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn tree_file_row<F, FR, FRev>(
    entry: &FileTreeEntry,
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
    let indent = (entry.depth as f32) * 14.0;
    let name_color = if reviewed { t.fg_faint } else { t.fg };
    let mut row = div()
        .id(("tree-file", ix))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(8.))
        .pl(px(10. + indent))
        .pr(px(10.))
        .h(px(28.))
        .bg(rgb(bg_row))
        .border_b_1()
        .border_color(rgb(t.row_border))
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
    row.child(status_dot(hunk))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(name_color))
                .child(SharedString::from(entry.name.clone())),
        )
        .into_any_element()
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
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(4.))
        .pl(px(10. + indent))
        .pr(px(10.))
        .h(px(28.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .cursor_pointer()
        .on_click(on_click)
        .child(icons::icon(chevron_glyph, 10., t.fg_faint))
        .child(icons::icon(glyph::FOLDER_SIMPLE, 12., t.fg_dim))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(entry.name.clone())),
        )
        .into_any_element()
}
