use std::collections::HashSet;

use gpui::{
    AnyElement, ClickEvent, Context, Div, InteractiveElement, IntoElement, ParentElement,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::{BookmarkInfo, ChangeInfo, ChecksStatus, DiffStats, PrInfo, PrState};

use super::RepoWindow;
use super::model::{active_bookmark_sync_label, working_copy_stat_label};
use crate::app::theme::{FONT_META, Theme};
use crate::diff::middle_elide;
use crate::ui::icons::{glyph, icon};

pub(super) fn leading_items(
    repo_path: SharedString,
    changes: &[ChangeInfo],
    bookmarks: &[BookmarkInfo],
    pr: Option<&PrInfo>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Vec<AnyElement> {
    let mut items = Vec::new();
    items.push(
        status_item_base("status-path", Some(glyph::FOLDER), repo_path, t)
            .max_w(px(420.))
            .into_any_element(),
    );
    if let Some(item) = active_bookmark_sync_item(changes, bookmarks, t) {
        items.push(item);
    }
    if let Some(pr) = pr {
        items.push(pr_link(pr, t, cx));
    }
    items
}

pub(super) fn trailing_items(
    changes: &[ChangeInfo],
    working_copy_stats: Option<&DiffStats>,
    operation: &str,
    selected: Option<usize>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Vec<AnyElement> {
    let mut items = Vec::new();
    if let Some(stats) = working_copy_stats.and_then(working_copy_stat_label) {
        items.push(status_item(
            "status-wc-stat",
            glyph::PENCIL_CIRCLE,
            stats,
            t,
        ));
    }

    let divergent_count = changes
        .iter()
        .filter(|change| change.is_divergent)
        .map(|change| change.change_id.id.as_str())
        .collect::<HashSet<_>>()
        .len();
    if divergent_count > 0 {
        items.push(status_action(
            "status-divergent",
            Some(glyph::GIT_BRANCH),
            format!("{divergent_count} divergent"),
            t,
            cx,
            |view, _, _, cx| {
                view.select_first_status_match(|change| change.is_divergent, cx);
            },
        ));
    }

    let conflict_count = changes.iter().filter(|change| change.has_conflict).count();
    if conflict_count > 0 {
        items.push(status_action(
            "status-conflicts",
            Some(glyph::WARNING),
            format!("{conflict_count} conflicted"),
            t,
            cx,
            |view, _, _, cx| {
                view.select_first_status_match(|change| change.has_conflict, cx);
            },
        ));
    }

    let operation = operation.trim();
    if !operation.is_empty() {
        items.push(status_action(
            "status-last-op",
            Some(glyph::ARROW_CLOCKWISE),
            middle_elide(operation, 40),
            t,
            cx,
            |view, _, _, cx| {
                view.open_operation_log(cx);
            },
        ));
    }

    let count = changes.len();
    let position_label = match selected {
        Some(ix) if count > 0 => format!("{} of {count}", ix + 1),
        _ if count > 0 => format!("{count} changes"),
        _ => "—".to_string(),
    };
    items.push(status_item(
        "status-changes",
        glyph::GIT_BRANCH,
        position_label,
        t,
    ));
    items
}

pub(super) fn status_group(items: Vec<AnyElement>, t: &Theme) -> AnyElement {
    let mut row = div().flex().flex_row().items_center().min_w_0();
    for (ix, item) in items.into_iter().enumerate() {
        if ix > 0 {
            row = row.child(separator(t));
        }
        row = row.child(item);
    }
    row.into_any_element()
}

fn separator(t: &Theme) -> AnyElement {
    div()
        .px(px(4.))
        .text_color(rgb(t.fg_faint))
        .child("·")
        .into_any_element()
}

fn status_item(
    id: &'static str,
    glyph_str: &'static str,
    text: impl Into<SharedString>,
    t: &Theme,
) -> AnyElement {
    status_item_base(id, Some(glyph_str), text, t).into_any_element()
}

fn status_action<F>(
    id: &'static str,
    glyph_str: Option<&'static str>,
    text: impl Into<SharedString>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
    on_click: F,
) -> AnyElement
where
    F: Fn(&mut RepoWindow, &ClickEvent, &mut Window, &mut Context<RepoWindow>) + 'static,
{
    status_item_base(id, glyph_str, text, t)
        .cursor_pointer()
        .hover(|style| style.text_color(rgb(t.fg)))
        .on_click(cx.listener(on_click))
        .into_any_element()
}

fn status_item_base(
    id: &'static str,
    glyph_str: Option<&'static str>,
    text: impl Into<SharedString>,
    t: &Theme,
) -> Stateful<Div> {
    let mut item = div()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_row()
        .items_center()
        .min_w_0()
        .gap(px(3.))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim));
    if let Some(glyph_str) = glyph_str {
        item = item.child(icon(glyph_str, 10., t.fg_dim));
    }
    item.child(div().min_w_0().truncate().child(text.into()))
}

fn active_bookmark_sync_item(
    changes: &[ChangeInfo],
    bookmarks: &[BookmarkInfo],
    t: &Theme,
) -> Option<AnyElement> {
    active_bookmark_sync_label(changes, bookmarks)
        .map(|text| status_item("status-bookmark-sync", glyph::BOOKMARK, text, t))
}

fn pr_link(pr: &PrInfo, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let glyph_str = match pr.checks {
        ChecksStatus::Passing => Some(glyph::CHECK),
        ChecksStatus::Failing => Some(glyph::X),
        ChecksStatus::Pending => Some(glyph::DOT),
        ChecksStatus::None => None,
    };
    let state = match pr.state {
        PrState::Open => "open",
        PrState::Closed => "closed",
        PrState::Merged => "merged",
    };
    let url = SharedString::from(pr.url.clone());
    status_action(
        "status-pr",
        glyph_str,
        format!("#{} {state}", pr.number),
        t,
        cx,
        move |_, _, _, cx| {
            cx.open_url(url.as_ref());
        },
    )
}
