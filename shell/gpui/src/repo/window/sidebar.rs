use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
    uniform_list,
};

use super::RepoWindow;
use super::dag::{DagRowLanes, dag_column};
use super::dag_row::{BookmarkDrop, BookmarkRightClick, DagRow, dag_row};
use crate::app::theme::{FONT_META, Theme};
use crate::ui::icons::glyph;
use crate::ui::primitives::{button, icon_label, no_scrollbar_gutter};

pub(super) fn sidebar(
    view: &RepoWindow,
    t: &Theme,
    width: f32,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let vm = view.vm.read(cx);
    let repo_open = vm.repo.is_some();
    let changes = vm.graph.changes.clone();
    let loading_more = vm.loading.more;
    let show_load_more = vm.error.is_none() && !changes.is_empty();

    let body: AnyElement = if !repo_open {
        div().into_any_element()
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
        let change_count = changes.len();
        let row_count = change_count + usize::from(show_load_more);
        let t_clone = t.clone();
        let scroll = view.scrolls.changes.clone();
        let changes_for_processor = changes.clone();
        let view_handle = cx.entity();
        let dag_layout = view.vm.read(cx).graph.dag_layout.clone();
        let entries = view.vm.read(cx).graph.entries.clone();
        let max_lanes = dag_layout.max_lanes();
        let list = uniform_list(
            "changes",
            row_count,
            cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                let t = t_clone.clone();
                let (selected, compare_source_change_id) = {
                    let vm = this.vm.read(cx);
                    (
                        vm.selected,
                        vm.compare
                            .as_ref()
                            .and_then(|compare| compare.source_change_id.clone()),
                    )
                };
                let view_handle = view_handle.clone();
                let dag_layout = dag_layout.clone();
                let entries = entries.clone();
                range
                    .map(|ix| {
                        if ix == change_count {
                            return load_more_button(loading_more, &t, cx);
                        }
                        let is_selected = selected == Some(ix);
                        let change = changes_for_processor[ix].clone();
                        let is_compare_source =
                            compare_source_change_id.as_deref() == Some(change.change_id.as_str());
                        let on_click = cx.listener(move |view, event: &ClickEvent, _window, cx| {
                            view.select_or_compare_change(ix, event.modifiers().shift, cx);
                        });
                        let change_for_menu = change.clone();
                        let on_right_click =
                            cx.listener(move |view, ev: &MouseDownEvent, _window, cx| {
                                let items = view.build_change_menu(&change_for_menu, cx);
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
                        let view_for_drop = view_handle.clone();
                        let drop_rev = crate::repo::revset::change_revision(&change);
                        let on_bookmark_drop: BookmarkDrop = std::sync::Arc::new(
                            move |name: &str, _w: &mut Window, cx: &mut App| {
                                let name = name.to_owned();
                                let rev = drop_rev.clone();
                                view_for_drop.update(cx, |view, cx| {
                                    view.drop_bookmark_on_rev(name, rev, cx);
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
                        let next_active_lanes = if ix + 1 < change_count {
                            dag_layout.active_lane_indices(ix + 1).to_vec()
                        } else {
                            Vec::new()
                        };
                        let dag_col = entries.get(ix).map(|entry| {
                            dag_column(
                                entry,
                                DagRowLanes {
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
                        dag_row(
                            DagRow {
                                change: &changes_for_processor[ix],
                                is_selected,
                                is_compare_source,
                                ix,
                                theme: &t,
                                dag_col,
                            },
                            on_click,
                            on_right_click,
                            on_bookmark,
                            on_bookmark_drop,
                        )
                    })
                    .collect()
            }),
        )
        .track_scroll(&scroll);
        no_scrollbar_gutter(list).h_full().into_any_element()
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
    col = col.child(div().flex_1().min_h_0().child(body));
    if show_commit_box {
        col = col.child(commit_box_editor(view, t, cx));
    }
    col.into_any_element()
}

fn commit_box_editor(view: &RepoWindow, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
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
        .child(view.summary_input.clone())
        .child(view.description_input.clone())
        .child(
            button("commit-working-copy", "Commit", t, true)
                .w_full()
                .on_click(cx.listener(|view, _: &gpui::ClickEvent, _w, cx| {
                    view.commit_working_copy_from_input(cx);
                })),
        )
        .into_any_element()
}

fn load_more_button(loading: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let label = if loading { "Loading…" } else { "Load more" };
    let mut button = div()
        .id(SharedString::from("load-more"))
        .flex()
        .flex_row()
        .w_full()
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
        .child(icon_label(glyph::ARROW_DOWN, label, 12., t.fg_dim));
    if !loading {
        button = button
            .cursor_pointer()
            .on_click(cx.listener(|view, _: &ClickEvent, _w, cx| view.load_more(cx)));
    }
    button.into_any_element()
}
