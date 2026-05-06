use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
    uniform_list,
};

use super::LogView;
use crate::app::theme::{FONT_META, Theme};
use crate::log::commit_row::{BookmarkRightClick, CommitRow, commit_box};

pub(super) fn sidebar(
    view: &LogView,
    t: &Theme,
    width: f32,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let vm = view.vm.read(cx);
    let error = vm.error.clone();
    let changes = vm.graph.changes.clone();
    let loading_more = vm.loading.more;
    let repo_path = vm.repo_path.clone();

    let body: AnyElement = if let Some(err) = error.clone() {
        if is_not_a_repo_error(&err) {
            no_repo_card(&repo_path, t)
        } else {
            div()
                .flex()
                .size_full()
                .items_center()
                .justify_center()
                .px_4()
                .text_color(rgb(t.error_fg))
                .child(err)
                .into_any_element()
        }
    } else if changes.is_empty() {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("No changes in default revset.")
            .into_any_element()
    } else {
        let count = changes.len();
        let t_clone = t.clone();
        let scroll = view.scrolls.changes.clone();
        let changes_for_processor = changes.clone();
        let view_handle = cx.entity();
        let dag_layout = view.vm.read(cx).graph.dag_layout.clone();
        let entries = view.vm.read(cx).graph.entries.clone();
        let max_lanes = dag_layout.max_lanes();
        let list = uniform_list(
            "changes",
            count,
            cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                let t = t_clone.clone();
                let selected = this.vm.read(cx).selected;
                let view_handle = view_handle.clone();
                let dag_layout = dag_layout.clone();
                let entries = entries.clone();
                range
                    .map(|ix| {
                        let is_selected = selected == Some(ix);
                        let change = changes_for_processor[ix].clone();
                        let on_click = cx.listener(move |view, _event, _window, cx| {
                            view.select_change(ix, cx);
                        });
                        let change_for_menu = change.clone();
                        let on_right_click =
                            cx.listener(move |view, ev: &MouseDownEvent, _window, cx| {
                                let items = LogView::build_change_menu(&change_for_menu);
                                view.open_context_menu(ev.position, items, cx);
                            });
                        let view_for_bm = view_handle.clone();
                        let on_bookmark: BookmarkRightClick = std::sync::Arc::new(
                            move |name: &str,
                                  ev: &MouseDownEvent,
                                  _w: &mut Window,
                                  cx: &mut App| {
                                let position = ev.position;
                                let name = name.to_owned();
                                view_for_bm.update(cx, |view, cx| {
                                    let items = view.build_bookmark_menu(&name, cx);
                                    view.open_context_menu(position, items, cx);
                                });
                            },
                        );
                        let row_lane = dag_layout.lane(&change.commit_id);
                        let active_lanes = dag_layout.active_lane_indices(ix).to_vec();
                        let prev_active_lanes = if ix > 0 {
                            dag_layout.active_lane_indices(ix - 1).to_vec()
                        } else {
                            Vec::new()
                        };
                        let next_active_lanes = if ix + 1 < count {
                            dag_layout.active_lane_indices(ix + 1).to_vec()
                        } else {
                            Vec::new()
                        };
                        let dag_col = entries.get(ix).map(|entry| {
                            crate::log::dag::dag_column(
                                entry,
                                crate::log::dag::DagRowLanes {
                                    row_lane,
                                    active_lanes,
                                    prev_active_lanes,
                                    next_active_lanes,
                                    max_lanes,
                                },
                                &dag_layout,
                                &t,
                            )
                        });
                        commit_box(
                            CommitRow {
                                change: &changes_for_processor[ix],
                                is_selected,
                                ix,
                                theme: &t,
                                dag_col,
                            },
                            on_click,
                            on_right_click,
                            on_bookmark,
                        )
                    })
                    .collect()
            }),
        )
        .track_scroll(&scroll);
        crate::ui::primitives::no_scrollbar_gutter(list)
            .h_full()
            .into_any_element()
    };

    let load_more = if error.is_none() && !changes.is_empty() {
        Some(load_more_button(loading_more, t, cx))
    } else {
        None
    };

    let bookmarks = view.vm.read(cx).graph.bookmarks.clone();
    let bookmark_bar_el = if bookmarks.is_empty() {
        None
    } else {
        Some(bookmark_bar(bookmarks, t, cx))
    };
    let show_commit_box = view
        .vm
        .read(cx)
        .selected_change()
        .map(|c| c.is_working_copy)
        .unwrap_or(false);

    let mut col = div()
        .flex()
        .flex_col()
        .w(px(width))
        .h_full()
        .bg(rgb(t.sidebar_bg));
    if let Some(bar) = bookmark_bar_el {
        col = col.child(bar);
    }
    col = col.child(div().flex_1().min_h_0().child(body));
    if let Some(button) = load_more {
        col = col.child(button);
    }
    if show_commit_box {
        col = col.child(commit_box_placeholder(t, cx));
    }
    col.into_any_element()
}

fn commit_box_placeholder(t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    let hover_bg = t.row_alt_bg;
    let active_bg = t.selected_bg;
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .px(px(10.))
        .py(px(8.))
        .border_t_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg))
        .child(
            div()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg_dim))
                .child("Commit working copy"),
        )
        .child(
            div()
                .id(SharedString::from("commit-placeholder"))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .w_full()
                .h(px(28.))
                .rounded_md()
                .bg(rgb(t.toggle_inactive_bg))
                .text_size(px(11.))
                .text_color(rgb(t.toggle_inactive_fg))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(hover_bg)))
                .active(|s| s.bg(rgb(active_bg)))
                .on_click(cx.listener(|view, _: &gpui::ClickEvent, _w, cx| {
                    view.show_coming_soon("Commit", cx);
                }))
                .child("Commit — coming soon"),
        )
        .into_any_element()
}

fn is_not_a_repo_error(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("no jj repo")
        || lower.contains("not a")
        || lower.contains("does not exist")
        || lower.contains("no working copy")
}

fn no_repo_card(repo_path: &SharedString, t: &Theme) -> AnyElement {
    let path = repo_path.clone();
    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap(px(12.))
        .px(px(24.))
        .child(crate::ui::icons::icon(
            crate::ui::icons::glyph::FOLDER,
            48.,
            t.fg_dim,
        ))
        .child(
            div()
                .text_size(px(15.))
                .text_color(rgb(t.fg))
                .child("Not a Jujutsu repository"),
        )
        .child(
            div()
                .max_w(px(360.))
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child(path),
        )
        .child(
            div()
                .max_w(px(360.))
                .text_size(px(12.))
                .text_color(rgb(t.fg_dim))
                .child(
                    "Run `jj git init` in this folder, or pass another path with --repo / drop a folder on the dock icon.",
                ),
        )
        .into_any_element()
}

fn bookmark_bar(
    bookmarks: std::sync::Arc<Vec<jayjay_core::BookmarkInfo>>,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let mut row = div()
        .id(SharedString::from("bookmark-bar"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(10.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .overflow_x_scroll();

    for bookmark in bookmarks.iter() {
        let name = bookmark.name.clone();
        let target_change_id = bookmark.change_id.clone();
        let chip = div()
            .id(SharedString::from(format!("bm-{name}")))
            .flex_none()
            .px(px(8.))
            .py(px(2.))
            .rounded_full()
            .bg(rgb(t.tag_bookmark_bg))
            .text_size(px(FONT_META))
            .text_color(rgb(t.tag_bookmark_fg))
            .cursor_pointer()
            .on_click(cx.listener(move |view, _: &ClickEvent, _w, cx| {
                view.reveal_change_id(&target_change_id, cx);
            }))
            .child(SharedString::from(bookmark.name.clone()));
        row = row.child(chip);
    }
    row.into_any_element()
}

fn load_more_button(loading: bool, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    let label = if loading { "Loading…" } else { "Load more" };
    let mut button = div()
        .id(SharedString::from("load-more"))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(6.))
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_t_1()
        .border_color(rgb(t.border))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(crate::ui::icons::icon(
            crate::ui::icons::glyph::ARROW_DOWN,
            12.,
            t.fg_dim,
        ))
        .child(label);
    if !loading {
        button = button
            .cursor_pointer()
            .on_click(cx.listener(|view, _: &ClickEvent, _w, cx| view.load_more(cx)));
    }
    button.into_any_element()
}
