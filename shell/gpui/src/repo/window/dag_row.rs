use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use gpui::{
    AnyElement, App, ClickEvent, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::ChangeInfo;

use crate::app::theme::{FONT_BODY, FONT_ID, FONT_META, FONT_TAG, Theme};
use crate::ui::primitives::capsule;

pub(super) type BookmarkRightClick =
    Arc<dyn Fn(&str, &MouseDownEvent, &mut Window, &mut App) + Send + Sync + 'static>;

/// Pure-data inputs for one DAG row in the sidebar.
pub(super) struct DagRow<'a> {
    pub change: &'a ChangeInfo,
    pub is_selected: bool,
    pub is_compare_source: bool,
    pub ix: usize,
    pub theme: &'a Theme,
    pub dag_col: Option<AnyElement>,
}

pub(super) fn dag_row<F, FR>(
    row: DagRow<'_>,
    on_click: F,
    on_right_click: FR,
    on_bookmark_right_click: BookmarkRightClick,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    FR: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
{
    let DagRow {
        change,
        is_selected,
        is_compare_source,
        ix,
        theme: t,
        dag_col,
    } = row;
    let short_id: SharedString = change.change_id.chars().take(12).collect::<String>().into();
    let short_commit: SharedString = change.commit_id.chars().take(12).collect::<String>().into();
    let summary = first_line(&change.description);

    let row_bg = if is_selected {
        t.selected_bg
    } else if is_compare_source {
        t.tag_divergent_bg
    } else {
        t.sidebar_bg
    };
    let hover_bg = if is_selected {
        t.selected_bg
    } else if is_compare_source {
        t.tag_divergent_bg
    } else {
        t.row_alt_bg
    };

    let mut row_div = div()
        .id(("change", ix))
        .flex()
        .flex_row()
        .w_full()
        .h(px(76.))
        .bg(rgb(row_bg))
        .hover(|s| s.bg(rgb(hover_bg)))
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click);
    if let Some(col) = dag_col {
        row_div = row_div.child(col);
    }
    row_div
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .pr_3()
                .py_2()
                .flex_1()
                .min_w_0()
                .child(tags_row(change, short_id, ix, t, on_bookmark_right_click))
                .child(summary_line(&summary, t))
                .child(meta_row(&change.author.name, short_commit, t)),
        )
        .into_any_element()
}

fn tags_row(
    change: &ChangeInfo,
    short_id: SharedString,
    row_ix: usize,
    t: &Theme,
    on_bookmark_right_click: BookmarkRightClick,
) -> impl IntoElement {
    let mut row = div().flex().flex_row().items_center().gap(px(5.)).child(
        div()
            .font_family(crate::app::fonts::mono())
            .text_size(px(FONT_ID))
            .text_color(rgb(if change.is_immutable { t.fg_dim } else { t.fg }))
            .child(short_id),
    );

    if change.is_working_copy {
        row = row.child(capsule("@", t.tag_wc_bg, t.tag_wc_fg, FONT_TAG));
    }
    if change.has_conflict {
        row = row.child(capsule(
            "conflict",
            t.tag_conflict_bg,
            t.tag_conflict_fg,
            FONT_TAG,
        ));
    }
    if change.is_divergent {
        row = row.child(capsule(
            "divergent",
            t.tag_divergent_bg,
            t.tag_divergent_fg,
            FONT_TAG,
        ));
    }
    for (b_ix, bm) in change.bookmarks.iter().take(3).enumerate() {
        let cb = on_bookmark_right_click.clone();
        row = row.child(bookmark_chip(row_ix, b_ix, bm.clone(), t, cb));
    }
    if change.bookmarks.len() > 3 {
        let extra = change.bookmarks.len() - 3;
        row = row.child(capsule(format!("+{extra}"), t.tag_bg, t.tag_fg, FONT_TAG));
    }
    if change.is_empty {
        row = row.child(capsule("empty", t.tag_bg, t.tag_fg, FONT_TAG));
    }
    row
}

fn bookmark_chip(
    row_ix: usize,
    b_ix: usize,
    name: String,
    t: &Theme,
    on_right_click: BookmarkRightClick,
) -> impl IntoElement {
    let label = SharedString::from(name.clone());
    div()
        .id(("bm", row_ix * 16 + b_ix))
        .flex_none()
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(t.tag_bookmark_bg))
        .text_color(rgb(t.tag_bookmark_fg))
        .text_size(px(FONT_TAG))
        .on_mouse_down(MouseButton::Right, move |ev, w, cx| {
            cx.stop_propagation();
            on_right_click(&name, ev, w, cx);
        })
        .child(label)
}

fn summary_line(summary: &str, t: &Theme) -> impl IntoElement {
    if summary.is_empty() {
        div()
            .text_size(px(FONT_BODY))
            .text_color(rgb(t.fg_faint))
            .truncate()
            .child("(no description)")
    } else {
        div()
            .text_size(px(FONT_BODY))
            .text_color(rgb(t.fg))
            .truncate()
            .child(SharedString::from(summary.to_owned()))
    }
}

fn meta_row(author: &str, short_commit: SharedString, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(author.to_owned()))
        .child(
            div()
                .font_family(crate::app::fonts::mono())
                .text_color(rgb(t.fg_faint))
                .child(short_commit),
        )
}

pub(super) fn format_when(ts_millis: i64) -> String {
    let dt: DateTime<Local> = match Local.timestamp_millis_opt(ts_millis).single() {
        Some(dt) => dt,
        None => return String::new(),
    };
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub(super) fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}
