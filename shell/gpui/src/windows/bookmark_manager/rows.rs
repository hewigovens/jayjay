use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb, uniform_list,
};
use jayjay_core::{BookmarkInfo, RemoteBookmarkTarget, RemoteSyncStatus};

use super::BookmarkManagerView;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::repo::window::split_prefix;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{capsule, no_scrollbar_gutter};

pub(super) fn bookmark_list(
    bookmarks: Arc<Vec<BookmarkInfo>>,
    t: &Theme,
    cx: &mut Context<BookmarkManagerView>,
) -> AnyElement {
    let count = bookmarks.len();
    let theme = Arc::new(t.clone());
    let list = uniform_list(
        "bookmark-manager",
        count,
        cx.processor(move |_this, range: std::ops::Range<usize>, _w, cx| {
            range
                .map(|ix| bookmark_row(bookmarks[ix].clone(), theme.clone(), cx))
                .collect()
        }),
    );
    no_scrollbar_gutter(list)
        .flex_1()
        .min_h_0()
        .w_full()
        .into_any_element()
}

fn bookmark_row(
    bookmark: BookmarkInfo,
    t: Arc<Theme>,
    cx: &mut Context<BookmarkManagerView>,
) -> AnyElement {
    let remote_suffix = bookmark
        .available_remotes
        .first()
        .filter(|_| !bookmark.has_local_target)
        .map(|remote| format!("@{remote}"))
        .unwrap_or_default();
    let name = format!("{}{remote_suffix}", bookmark.name);
    let row_selector = format!("bookmark-row-{}", bookmark.name);
    let bookmark_for_menu = bookmark.clone();

    div()
        .id(SharedString::from(row_selector.clone()))
        .debug_selector(move || row_selector.clone())
        .flex()
        .flex_row()
        .w_full()
        .items_center()
        .gap(px(10.))
        .px(px(16.))
        .py(px(8.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |view, ev: &MouseDownEvent, _window, cx| {
                view.open_context_menu(ev.position, bookmark_for_menu.clone(), cx);
            }),
        )
        .child(status_icon(&bookmark, &t))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .min_w_0()
                .flex_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .font_family(fonts::mono())
                                .text_size(px(13.))
                                .text_color(rgb(t.fg))
                                .child(SharedString::from(name)),
                        )
                        .children(status_chips(&bookmark, &t)),
                )
                .child(bookmark_meta(&bookmark, &t)),
        )
        .children((!bookmark.change_id.is_empty()).then(|| change_id(&bookmark, &t)))
        .into_any_element()
}

fn status_icon(bookmark: &BookmarkInfo, t: &Theme) -> AnyElement {
    let color = if bookmark.is_deleted {
        t.file_removed_color
    } else if bookmark.is_conflicted {
        t.tag_divergent_fg
    } else if !bookmark.has_local_target || bookmark.is_tracking_remote {
        t.selected_accent
    } else {
        t.fg_dim
    };
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(16.))
        .child(icons::icon(glyph::BOOKMARK, 13., color))
        .into_any_element()
}

fn bookmark_meta(bookmark: &BookmarkInfo, t: &Theme) -> AnyElement {
    let mut meta = div().flex().flex_row().items_center().gap(px(6.)).min_w_0();
    if !bookmark.description.trim().is_empty() {
        meta = meta.child(
            div()
                .min_w_0()
                .text_ellipsis()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(bookmark.description.clone())),
        );
    }
    for target in &bookmark.remote_targets {
        meta = meta.child(remote_badge(target, t));
    }
    meta.into_any_element()
}

fn status_chips(bookmark: &BookmarkInfo, t: &Theme) -> Vec<AnyElement> {
    let mut chips = Vec::new();
    if bookmark.is_deleted {
        chips.push(capsule("deleted", t.tag_removed_bg, t.tag_removed_fg, 9.).into_any_element());
    }
    if bookmark.is_conflicted {
        chips.push(
            capsule("conflicted", t.tag_conflict_bg, t.tag_conflict_fg, 9.).into_any_element(),
        );
    }
    if !bookmark.has_local_target && !bookmark.is_deleted {
        chips.push(
            capsule("remote-only", t.toggle_active_bg, t.toggle_active_fg, 9.).into_any_element(),
        );
    } else if bookmark.has_local_target && !bookmark.is_tracking_remote && !bookmark.is_deleted {
        chips.push(capsule("local", t.tag_bg, t.tag_fg, 9.).into_any_element());
    }
    chips
}

fn remote_badge(target: &RemoteBookmarkTarget, t: &Theme) -> AnyElement {
    let (label, bg, fg) = match target.status {
        RemoteSyncStatus::Synced => (
            format!("{} ✓", target.remote),
            t.tag_bookmark_bg,
            t.tag_bookmark_icon,
        ),
        RemoteSyncStatus::Ahead => (
            format!("ahead of {}", target.remote),
            t.tag_divergent_bg,
            t.tag_divergent_fg,
        ),
        RemoteSyncStatus::Behind => (
            format!("behind {}", target.remote),
            t.tag_divergent_bg,
            t.tag_divergent_fg,
        ),
        RemoteSyncStatus::Diverged => (
            format!("diverged from {}", target.remote),
            t.tag_conflict_bg,
            t.tag_conflict_fg,
        ),
    };
    capsule(label, bg, fg, 9.).into_any_element()
}

fn change_id(bookmark: &BookmarkInfo, t: &Theme) -> AnyElement {
    let shown: String = bookmark.change_id.id.chars().take(12).collect();
    let (prefix, rest) = split_prefix(&shown, bookmark.change_id.short_len);
    div()
        .flex_none()
        .flex()
        .font_family(fonts::mono())
        .text_size(px(11.))
        .child(
            div()
                .text_color(rgb(t.change_id_prefix))
                .child(SharedString::from(prefix)),
        )
        .child(
            div()
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(rest)),
        )
        .into_any_element()
}
