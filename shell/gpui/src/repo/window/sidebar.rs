use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
    uniform_list,
};

use super::RepoWindow;
use super::dag::{DagRowLanes, dag_column};
use super::dag_row::{BookmarkRightClick, DagDrop, DagRow, dag_row};
use super::revset_filter::revset_filter_panel;
use crate::app::fonts;
use crate::app::theme::{FONT_META, Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::primitives::{button, icon_button, icon_label, no_scrollbar_gutter, text_tooltip};

pub(super) fn sidebar(
    view: &RepoWindow,
    t: &Theme,
    width: f32,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (repo_open, changes, loading_more, show_load_more, default_revset, bookmarks) = {
        let vm = view.vm.read(cx);
        (
            vm.repo.is_some(),
            vm.graph.changes.clone(),
            vm.loading.more,
            vm.error.is_none() && vm.can_load_more && !vm.graph.changes.is_empty(),
            vm.revset_is_default(),
            vm.graph.bookmarks.clone(),
        )
    };

    let body: AnyElement = if !repo_open {
        div().into_any_element()
    } else if changes.is_empty() {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child(if default_revset {
                "No changes in default revset."
            } else {
                "No changes match this revset."
            })
            .into_any_element()
    } else {
        let change_count = changes.len();
        let row_count = change_count + usize::from(show_load_more);
        let t_clone = t.clone();
        let scroll = view.scrolls.changes.clone();
        let changes_for_processor = changes.clone();
        let bookmarks_for_processor = bookmarks.clone();
        let view_handle = cx.entity();
        let dag_layout = view.vm.read(cx).graph.dag_layout.clone();
        let entries = view.vm.read(cx).graph.entries.clone();
        let list = uniform_list(
            "changes",
            row_count,
            cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                let t = t_clone.clone();
                let (selected, selected_changes, compare_source_change_id) = {
                    let vm = this.vm.read(cx);
                    (
                        vm.selected,
                        vm.selected_change_indices(),
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
                        let has_multiple_selection = selected_changes.len() > 1;
                        let is_selected_change = if has_multiple_selection {
                            selected_changes.contains(&ix)
                        } else {
                            selected == Some(ix)
                        };
                        let change = changes_for_processor[ix].clone();
                        let is_compare_source = compare_source_change_id.as_deref()
                            == Some(change.change_id.as_str())
                            && (!has_multiple_selection || is_selected_change);
                        let is_selected = is_selected_change && !is_compare_source;
                        let on_click = cx.listener(move |view, event: &ClickEvent, _window, cx| {
                            view.handle_change_row_click(ix, event.modifiers(), cx);
                        });
                        let change_for_menu = change.clone();
                        let on_right_click =
                            cx.listener(move |view, ev: &MouseDownEvent, _window, cx| {
                                let items = view.build_change_menu(&change_for_menu, cx);
                                view.open_context_menu(ev.position, items, cx);
                            });
                        let view_for_bm = view_handle.clone();
                        let bookmark_rev = change.commit_id.id.clone();
                        let on_bookmark: BookmarkRightClick = std::sync::Arc::new(
                            move |name: &str,
                                  ev: &MouseDownEvent,
                                  _w: &mut Window,
                                  cx: &mut App| {
                                let position = ev.position;
                                let name = name.to_owned();
                                let rev = bookmark_rev.clone();
                                view_for_bm.update(cx, |view, cx| {
                                    let items = view.build_bookmark_menu(&name, Some(&rev), cx);
                                    view.open_context_menu(position, items, cx);
                                });
                            },
                        );
                        let view_for_drop = view_handle.clone();
                        let drop_target = change.clone();
                        let on_drop: DagDrop =
                            std::sync::Arc::new(move |drag, _window: &mut Window, cx: &mut App| {
                                let drag = drag.clone();
                                let destination = drop_target.clone();
                                view_for_drop.update(cx, |view, cx| {
                                    view.drop_dag_drag_on_change(drag, destination, cx);
                                });
                            });
                        let row_lane = dag_layout.lane(&change.commit_id);
                        let pass_through_lanes = dag_layout.pass_through_lane_indices(ix).to_vec();
                        let prev_active_lanes = if ix > 0 {
                            dag_layout.active_lane_indices(ix - 1).to_vec()
                        } else {
                            Vec::new()
                        };
                        let has_overflow = dag_layout.row_has_overflow(ix);
                        let has_missing_ancestry = dag_layout.row_has_missing_ancestry(ix);
                        let dag_col = entries.get(ix).map(|entry| {
                            dag_column(
                                entry,
                                DagRowLanes {
                                    row_lane,
                                    pass_through_lanes,
                                    prev_active_lanes,
                                    has_overflow,
                                    has_missing_ancestry,
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
                                bookmarks: bookmarks_for_processor.as_ref(),
                                entries: &entries,
                            },
                            on_click,
                            on_right_click,
                            on_bookmark,
                            on_drop,
                        )
                    })
                    .collect()
            }),
        )
        .track_scroll(&scroll);
        no_scrollbar_gutter(list).h_full().into_any_element()
    };

    let show_commit_box = {
        let vm = view.vm.read(cx);
        vm.selected_change()
            .is_some_and(|change| change.is_working_copy)
            && !vm.has_multiple_change_selection()
    };

    let mut col = div()
        .flex()
        .flex_col()
        .w(px(width))
        .h_full()
        .bg(rgb(t.sidebar_bg));
    if let Some(filter) = revset_filter_panel(view, t, cx) {
        col = col.child(filter);
    }
    if let Some(banner) = push_follow_up_banner(view, t, cx) {
        col = col.child(banner);
    }
    col = col.child(div().flex_1().min_h_0().child(body));
    if show_commit_box {
        col = col.child(commit_box_editor(view, t, cx));
    }
    col.into_any_element()
}

fn push_follow_up_banner(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    let bookmark = view.feedback.pending_push_bookmark.clone()?;
    let mut push = button("pending-push-confirm", "Push", t, true)
        .debug_selector(|| "pending-push-confirm".to_owned());
    if view.sync_activity.pushing {
        push = push.opacity(0.45);
    } else {
        push = push.on_click(cx.listener(|view, _: &ClickEvent, _window, cx| {
            view.confirm_pending_push(cx);
        }));
    }

    Some(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(12.))
            .py(px(8.))
            .border_b_1()
            .border_color(rgb(t.row_border))
            .bg(rgb(t.sidebar_bg))
            .text_size(ui_font_size(FONT_META))
            .text_color(rgb(t.fg_dim))
            .debug_selector(|| "pending-push-banner".to_owned())
            .child(icon_label(
                glyph::BOOKMARK,
                "Moved",
                12.,
                t.tag_bookmark_icon,
            ))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .font_family(fonts::mono())
                    .text_color(rgb(t.fg))
                    .child(bookmark),
            )
            .child(div().flex_1())
            .child(push)
            .child(
                icon_button("pending-push-dismiss", glyph::X, 12., 24., 24., t.fg_dim, t)
                    .debug_selector(|| "pending-push-dismiss".to_owned())
                    .tooltip(text_tooltip("Dismiss"))
                    .on_click(cx.listener(|view, _: &ClickEvent, _window, cx| {
                        view.dismiss_pending_push(cx);
                    })),
            )
            .into_any_element(),
    )
}

fn commit_box_editor(view: &RepoWindow, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .px(px(12.))
        .py(px(12.))
        .border_t_1()
        .border_color(rgb(t.row_border))
        .bg(rgb(t.sidebar_bg))
        .debug_selector(|| "commit-box-editor".to_owned())
        .child(view.summary_input.clone())
        .child(view.description_input.clone())
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .gap(px(8.))
                .child(super::commit_ai::generate_button(view, t, cx))
                .child(commit_box_button(
                    "describe-working-copy",
                    "Describe",
                    false,
                    "Save description (jj describe)",
                    RepoWindow::describe_working_copy_from_input,
                    t,
                    cx,
                ))
                .child(
                    commit_box_button(
                        "commit-working-copy",
                        "Commit",
                        true,
                        "Describe + start new change (jj commit)",
                        RepoWindow::commit_working_copy_from_input,
                        t,
                        cx,
                    )
                    .min_w(px(76.)),
                ),
        )
        .into_any_element()
}

fn commit_box_button(
    id: &'static str,
    label: &'static str,
    primary: bool,
    tooltip: &'static str,
    handler: fn(&mut RepoWindow, &mut Context<RepoWindow>),
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> gpui::Stateful<gpui::Div> {
    button(id, label, t, primary)
        .debug_selector(move || id.to_owned())
        .tooltip(text_tooltip(tooltip))
        .on_click(cx.listener(move |view, _: &ClickEvent, _w, cx| handler(view, cx)))
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
        .text_size(ui_font_size(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(icon_label(glyph::ARROW_DOWN, label, 12., t.fg_dim));
    if !loading {
        button = button
            .cursor_pointer()
            .on_click(cx.listener(|view, _: &ClickEvent, _w, cx| view.load_more(cx)));
    }
    button.into_any_element()
}
