use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use gpui::{
    AnyElement, App, AppContext, ClickEvent, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div,
    px, rgb,
};
use jayjay_core::{BookmarkInfo, ChangeInfo, CommitAuthor, GraphEntry};

use crate::app::theme::{FONT_BODY, FONT_ID, FONT_META, FONT_TAG, Theme};
use crate::ui::icons::glyph;
use crate::ui::primitives::{capsule, icon_chip};

use super::dag_drag::{DagDrag, DagDragGhost};

pub(super) type BookmarkRightClick =
    Arc<dyn Fn(&str, &MouseDownEvent, &mut Window, &mut App) + Send + Sync + 'static>;

/// Invoked when a dragged DAG reference or change is dropped onto this row.
pub(super) type DagDrop = Arc<dyn Fn(&DagDrag, &mut Window, &mut App) + 'static>;

/// Pure-data inputs for one DAG row in the sidebar.
pub(super) struct DagRow<'a> {
    pub change: &'a ChangeInfo,
    pub is_selected: bool,
    pub is_compare_source: bool,
    pub ix: usize,
    pub theme: &'a Theme,
    pub dag_col: Option<AnyElement>,
    pub bookmarks: &'a [BookmarkInfo],
    pub entries: &'a Arc<Vec<GraphEntry>>,
}

pub(super) fn dag_row<F, FR>(
    row: DagRow<'_>,
    on_click: F,
    on_right_click: FR,
    on_bookmark_right_click: BookmarkRightClick,
    on_drop: DagDrop,
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
        bookmarks,
        entries,
    } = row;
    let short_id: SharedString = change.change_id.chars().take(12).collect::<String>().into();
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

    let row_selector = format!("dag-change-{}", change.commit_id.id);
    let drop_ring = t.toggle_active_bg;
    let refused_drop = t.tag_conflict_fg;
    let drop_target = change.clone();
    let mut row_div = div()
        .id(("change", ix))
        .debug_selector(move || row_selector.clone())
        .flex()
        .flex_row()
        .w_full()
        .h(px(76.))
        .bg(rgb(row_bg))
        .hover(|s| s.bg(rgb(hover_bg)))
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click)
        .drag_over::<DagDrag>(move |style, drag, _, _| {
            if !drag.can_drop_on(&drop_target) {
                return style;
            }
            let color = if matches!(drag, DagDrag::WorkingCopy) && drop_target.is_immutable {
                refused_drop
            } else {
                drop_ring
            };
            style.bg(gpui::rgba(((color as u64) << 8) as u32 | 0x44))
        })
        .on_drop(move |drag: &DagDrag, w, cx| {
            on_drop(drag, w, cx);
        });
    if let Some(drag) = DagDrag::for_change(ix, entries) {
        row_div = row_div.on_drag(drag, move |drag: &DagDrag, _offset, _window, cx| {
            cx.new(|_| DagDragGhost::new(drag.clone()))
        });
    }
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
                .child(tags_row(
                    change,
                    short_id,
                    ix,
                    t,
                    bookmarks,
                    on_bookmark_right_click,
                ))
                .child(summary_line(&summary, t))
                .child(meta_row(&change.author, t)),
        )
        .into_any_element()
}

fn tags_row(
    change: &ChangeInfo,
    short_id: SharedString,
    row_ix: usize,
    t: &Theme,
    bookmarks: &[BookmarkInfo],
    on_bookmark_right_click: BookmarkRightClick,
) -> impl IntoElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .child(change_id_cell(
            &short_id,
            change.change_id.short_len,
            change.is_immutable,
            t,
        ));

    if change.is_working_copy {
        row = row.child(working_copy_chip(row_ix, t));
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
        row = row.child(bookmark_chip(
            row_ix,
            b_ix,
            bm.clone(),
            BookmarkInfo::is_conflicted_name(bookmarks, bm),
            t,
            cb,
        ));
    }
    if change.bookmarks.len() > 3 {
        let extra = change.bookmarks.len() - 3;
        row = row.child(capsule(format!("+{extra}"), t.tag_bg, t.tag_fg, FONT_TAG));
    }
    for tg in change.tags.iter().take(3) {
        row = row.child(tag_chip(tg.clone(), t));
    }
    if change.tags.len() > 3 {
        let extra = change.tags.len() - 3;
        row = row.child(capsule(
            format!("+{extra}"),
            t.tag_tag_bg,
            t.tag_tag_fg,
            FONT_TAG,
        ));
    }
    for ws in change.workspaces.iter().take(3) {
        row = row.child(capsule(
            format!("{ws}@"),
            t.tag_wc_bg,
            t.tag_wc_fg,
            FONT_TAG,
        ));
    }
    if change.workspaces.len() > 3 {
        let extra = change.workspaces.len() - 3;
        row = row.child(capsule(
            format!("+{extra}"),
            t.tag_wc_bg,
            t.tag_wc_fg,
            FONT_TAG,
        ));
    }
    if change.is_empty {
        row = row.child(capsule("empty", t.tag_bg, t.tag_fg, FONT_TAG));
    }
    row
}

/// The change id with its shortest unique prefix highlighted and the rest dimmed.
fn change_id_cell(
    short_id: &SharedString,
    short_len: u32,
    is_immutable: bool,
    t: &Theme,
) -> impl IntoElement {
    let (prefix, rest) = split_prefix(short_id.as_ref(), short_len);
    let prefix_color = if is_immutable {
        t.fg_dim
    } else {
        t.change_id_prefix
    };
    div()
        .flex()
        .flex_row()
        .font_family(crate::app::fonts::mono())
        .text_size(px(FONT_ID))
        .child(
            div()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(prefix_color))
                .child(SharedString::from(prefix)),
        )
        .child(
            div()
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(rest)),
        )
}

fn bookmark_chip(
    row_ix: usize,
    b_ix: usize,
    name: String,
    conflicted: bool,
    t: &Theme,
    on_right_click: BookmarkRightClick,
) -> impl IntoElement {
    let drag_name = name.clone();
    let debug_name = name.clone();
    let (icon, bg, fg, icon_color) = if conflicted {
        (
            glyph::WARNING,
            t.tag_divergent_bg,
            t.tag_divergent_fg,
            t.tag_divergent_fg,
        )
    } else {
        (
            glyph::BOOKMARK,
            t.tag_bookmark_bg,
            t.tag_bookmark_fg,
            t.tag_bookmark_icon,
        )
    };
    icon_chip(icon, name.clone(), bg, fg, icon_color, FONT_TAG)
    .id(("bm", row_ix * 16 + b_ix))
    .debug_selector(move || format!("dag-bookmark-{debug_name}"))
    .cursor_move()
    // Drag the chip onto another change to move the bookmark there.
    .on_drag(
        DagDrag::Bookmark {
            name: drag_name.clone(),
            conflicted,
        },
        move |drag: &DagDrag, _offset, _w, cx| {
            cx.new(|_| DagDragGhost::new(drag.clone()))
        },
    )
    .on_mouse_down(MouseButton::Right, move |ev, w, cx| {
        cx.stop_propagation();
        on_right_click(&name, ev, w, cx);
    })
}

fn working_copy_chip(row_ix: usize, t: &Theme) -> impl IntoElement {
    div()
        .id(("wc", row_ix))
        .debug_selector(|| "dag-working-copy".to_owned())
        .cursor_move()
        .on_drag(
            DagDrag::WorkingCopy,
            move |drag: &DagDrag, _offset, _w, cx| cx.new(|_| DagDragGhost::new(drag.clone())),
        )
        .child(capsule("@", t.tag_wc_bg, t.tag_wc_fg, FONT_TAG))
}

/// A git-tag chip: neutral pill with a colored tag glyph. Non-interactive
/// (tags have no row actions, unlike bookmarks).
fn tag_chip(name: String, t: &Theme) -> impl IntoElement {
    icon_chip(
        glyph::TAG,
        name,
        t.tag_tag_bg,
        t.tag_tag_fg,
        t.tag_tag_icon,
        FONT_TAG,
    )
}

fn summary_line(summary: &str, t: &Theme) -> impl IntoElement {
    if summary.is_empty() {
        div()
            .font_family(crate::app::fonts::mono())
            .text_size(px(FONT_BODY))
            .text_color(rgb(t.fg_faint))
            .truncate()
            .child("(no description)")
    } else {
        div()
            .font_family(crate::app::fonts::mono())
            .text_size(px(FONT_BODY))
            .text_color(rgb(t.fg))
            .truncate()
            .child(SharedString::from(summary.to_owned()))
    }
}

fn meta_row(author: &CommitAuthor, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(crate::ui::avatar::element(&author.email, &author.name, 14.))
        .child(SharedString::from(author.name.clone()))
        .child(
            div()
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(format_relative(author.timestamp_millis))),
        )
}

pub(super) fn format_when(ts_millis: i64) -> String {
    let dt: DateTime<Local> = match Local.timestamp_millis_opt(ts_millis).single() {
        Some(dt) => dt,
        None => return String::new(),
    };
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Split `value` into its shortest-unique-prefix and the remainder at `short_len`.
pub(crate) fn split_prefix(value: &str, short_len: u32) -> (String, String) {
    let n = (short_len as usize).min(value.chars().count());
    (
        value.chars().take(n).collect(),
        value.chars().skip(n).collect(),
    )
}

/// "10 days ago" — coarse relative age for the DAG meta line.
pub(crate) fn format_relative(ts_millis: i64) -> String {
    let dt: DateTime<Local> = match Local.timestamp_millis_opt(ts_millis).single() {
        Some(dt) => dt,
        None => return String::new(),
    };
    // Floor to whole minutes so a fresh change reads "1 minute ago" instead of ticking
    // per second; clamping also reads a clock-skewed future timestamp as "1 minute ago".
    let secs = Local::now().signed_duration_since(dt).num_seconds().max(60);
    let ago = |n: i64, unit: &str| {
        if n == 1 {
            format!("1 {unit} ago")
        } else {
            format!("{n} {unit}s ago")
        }
    };
    match secs {
        s if s < 3600 => ago(s / 60, "minute"),
        s if s < 86_400 => ago(s / 3600, "hour"),
        s if s < 604_800 => ago(s / 86_400, "day"),
        s if s < 2_592_000 => ago(s / 604_800, "week"),
        s if s < 31_536_000 => ago(s / 2_592_000, "month"),
        s => ago(s / 31_536_000, "year"),
    }
}

pub(super) fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::format_relative;
    use chrono::Local;

    #[test]
    fn format_relative_floors_sub_minute_to_one_minute() {
        let now = Local::now().timestamp_millis();
        // Fresh and sub-minute changes read as whole minutes, never per-second.
        assert_eq!(format_relative(now), "1 minute ago");
        assert_eq!(format_relative(now - 30_000), "1 minute ago");
        // Coarser units are unaffected.
        assert_eq!(format_relative(now - 2 * 3_600_000), "2 hours ago");
    }
}
