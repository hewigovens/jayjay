use std::collections::HashSet;

use gpui::{
    AnyElement, ClickEvent, Context, Div, InteractiveElement, IntoElement, ParentElement,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::{BookmarkInfo, ChangeInfo, ChecksStatus, DiffStats, PrInfo, PrState};

use super::RepoWindow;
use super::model::{active_bookmark_sync_item_data, working_copy_stat_label};
use crate::app::theme::{FONT_META, Theme, ui_font_size};
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::{dot_separator, text_tooltip};

struct StatusItemSpec {
    id: &'static str,
    glyph: Option<&'static str>,
    text: SharedString,
    tooltip: Option<SharedString>,
    shrinks: bool,
}

impl StatusItemSpec {
    fn new(
        id: &'static str,
        glyph: Option<&'static str>,
        text: impl Into<SharedString>,
        tooltip: Option<SharedString>,
        shrinks: bool,
    ) -> Self {
        Self {
            id,
            glyph,
            text: text.into(),
            tooltip,
            shrinks,
        }
    }
}

pub(super) fn leading_items(
    repo_path: SharedString,
    changes: &[ChangeInfo],
    bookmarks: &[BookmarkInfo],
    pr: Option<&PrInfo>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Vec<AnyElement> {
    let mut items = Vec::new();
    items.push(status_action(
        StatusItemSpec::new(
            "status-path",
            Some(glyph::FOLDER),
            repo_path,
            Some(crate::platform::SHOW_IN_FILE_MANAGER_LABEL.into()),
            true,
        ),
        t,
        cx,
        |view, _, _, cx| {
            view.show_repo_in_file_manager(cx);
        },
    ));
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
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Vec<AnyElement> {
    let mut items = Vec::new();
    let operation = operation.trim();
    if !operation.is_empty() {
        items.push(status_action(
            StatusItemSpec::new(
                "status-last-op",
                Some(glyph::ARROW_CLOCKWISE),
                operation.to_owned(),
                None,
                true,
            ),
            t,
            cx,
            |view, _, _, cx| {
                view.open_operation_log(cx);
            },
        ));
    }

    if let Some(stats) = working_copy_stats.and_then(working_copy_stat_label) {
        items.push(
            status_item_base(
                StatusItemSpec::new(
                    "status-wc-stat",
                    Some(glyph::PENCIL),
                    stats,
                    Some("Working-copy changes".into()),
                    false,
                ),
                t,
            )
            .into_any_element(),
        );
    }

    let divergent_count = changes
        .iter()
        .filter(|change| change.is_divergent)
        .map(|change| change.change_id.id.as_str())
        .collect::<HashSet<_>>()
        .len();
    if divergent_count > 0 {
        items.push(status_action(
            StatusItemSpec::new(
                "status-divergent",
                Some(glyph::GIT_BRANCH),
                format!("{divergent_count} divergent"),
                None,
                false,
            ),
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
            StatusItemSpec::new(
                "status-conflicts",
                Some(glyph::WARNING),
                format!("{conflict_count} conflicted"),
                None,
                false,
            ),
            t,
            cx,
            |view, _, _, cx| {
                view.select_first_status_match(|change| change.has_conflict, cx);
            },
        ));
    }
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
    dot_separator(t).px(px(4.)).into_any_element()
}

fn status_action<F>(
    spec: StatusItemSpec,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
    on_click: F,
) -> AnyElement
where
    F: Fn(&mut RepoWindow, &ClickEvent, &mut Window, &mut Context<RepoWindow>) + 'static,
{
    let spec = StatusItemSpec {
        tooltip: Some(spec.tooltip.unwrap_or_else(|| spec.text.clone())),
        ..spec
    };
    status_item_base(spec, t)
        .cursor_pointer()
        .hover(|style| style.text_color(rgb(t.fg)))
        .on_click(cx.listener(on_click))
        .into_any_element()
}

fn status_item_base(spec: StatusItemSpec, t: &Theme) -> Stateful<Div> {
    let id = spec.id;
    let mut item = div()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_row()
        .items_center()
        .min_w_0()
        .gap(px(3.))
        .text_size(ui_font_size(FONT_META))
        .text_color(rgb(t.fg_dim));
    if spec.shrinks {
        item = item.flex_shrink(1.);
    } else {
        item = item.flex_none();
    }
    if let Some(tooltip) = spec.tooltip {
        item = item.tooltip(text_tooltip(tooltip));
    }
    if let Some(glyph_str) = spec.glyph {
        item = item.child(icon(glyph_str, 10., t.fg_dim));
    }
    item.child(div().min_w_0().truncate().child(spec.text))
}

fn active_bookmark_sync_item(
    changes: &[ChangeInfo],
    bookmarks: &[BookmarkInfo],
    t: &Theme,
) -> Option<AnyElement> {
    active_bookmark_sync_item_data(changes, bookmarks).map(|(text, tooltip)| {
        status_item_base(
            StatusItemSpec::new(
                "status-bookmark-sync",
                Some(glyph::BOOKMARK),
                text,
                Some(tooltip.into()),
                false,
            ),
            t,
        )
        .into_any_element()
    })
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
        StatusItemSpec::new(
            "status-pr",
            glyph_str,
            format!("#{} {state}", pr.number),
            Some(pr.title.clone().into()),
            false,
        ),
        t,
        cx,
        move |_, _, _, cx| {
            crate::app::links::open_url(cx, url.as_ref());
        },
    )
}
