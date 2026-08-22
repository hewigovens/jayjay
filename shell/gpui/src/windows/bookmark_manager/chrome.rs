use gpui::{
    AnyElement, ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::BookmarkInfo;

use super::BookmarkManagerView;
use crate::app::theme::{Theme, with_alpha};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{button, checkbox_row};
use crate::ui::text_area::TextArea;

#[derive(Clone, Copy, Default)]
pub(super) struct BookmarkStats {
    active: usize,
    deleted: usize,
    conflicted: usize,
    local_only: usize,
    remote_only: usize,
}

impl BookmarkStats {
    pub(super) fn from_bookmarks(bookmarks: &[BookmarkInfo]) -> Self {
        let mut stats = Self::default();
        for bookmark in bookmarks {
            if bookmark.is_deleted {
                stats.deleted += 1;
            } else {
                stats.active += 1;
                stats.local_only +=
                    usize::from(bookmark.has_local_target && !bookmark.is_tracking_remote);
                stats.remote_only += usize::from(!bookmark.has_local_target);
            }
            stats.conflicted += usize::from(bookmark.is_conflicted);
        }
        stats
    }
}

pub(super) fn header(filter: Entity<TextArea>, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .px(px(16.))
        .py(px(12.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(glyph::BOOKMARK, 15., t.fg))
        .child(
            div()
                .text_size(px(15.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("Bookmark Manager"),
        )
        .child(div().flex_1())
        .child(icons::icon(glyph::SEARCH, 14., t.fg_dim))
        .child(
            div()
                .id("bookmark-filter")
                .debug_selector(|| "bookmark-filter".to_owned())
                .w(px(180.))
                .child(filter),
        )
        .into_any_element()
}

pub(super) fn stats_bar(
    stats: BookmarkStats,
    show_deleted: bool,
    t: &Theme,
    cx: &mut Context<BookmarkManagerView>,
) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .px(px(16.))
        .py(px(8.))
        .bg(rgba(with_alpha(t.fg, if t.is_dark { 0x10 } else { 0x08 })))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(stat_badge("active", stats.active, t.file_added_color))
        .children(
            (stats.deleted > 0).then(|| stat_badge("deleted", stats.deleted, t.file_removed_color)),
        )
        .children(
            (stats.conflicted > 0)
                .then(|| stat_badge("conflicted", stats.conflicted, t.tag_divergent_fg)),
        )
        .children(
            (stats.local_only > 0).then(|| stat_badge("local-only", stats.local_only, t.fg_dim)),
        )
        .children(
            (stats.remote_only > 0)
                .then(|| stat_badge("remote-only", stats.remote_only, t.selected_accent)),
        )
        .child(div().flex_1())
        .child(
            checkbox_row("bookmark-show-deleted", "Show deleted", show_deleted, t).on_click(
                cx.listener(|view, _: &ClickEvent, _, cx| {
                    view.toggle_deleted(cx);
                }),
            ),
        )
        .into_any_element()
}

pub(super) fn footer(t: &Theme, cx: &mut Context<BookmarkManagerView>) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .justify_end()
        .px(px(16.))
        .py(px(10.))
        .border_t_1()
        .border_color(rgb(t.border))
        .child(
            button("bookmark-manager-done", "Done", t, false)
                .debug_selector(|| "bookmark-manager-done".to_owned())
                .on_click(cx.listener(|_, _: &ClickEvent, window, _| {
                    window.remove_window();
                })),
        )
        .into_any_element()
}

fn stat_badge(label: &'static str, count: usize, color: u32) -> AnyElement {
    let selector = SharedString::from(format!("bookmark-stat-{label}-{count}"));
    let debug_selector = selector.clone();
    div()
        .id(selector)
        .debug_selector(move || debug_selector.to_string())
        .flex_none()
        .px(px(8.))
        .py(px(3.))
        .rounded_full()
        .bg(rgba(with_alpha(color, 0x18)))
        .text_size(px(10.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(color))
        .child(SharedString::from(format!("{count} {label}")))
        .into_any_element()
}

pub(super) fn placeholder(message: &str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .text_size(px(12.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(message.to_owned()))
        .into_any_element()
}

pub(super) fn placeholder_err(message: &SharedString, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .px(px(24.))
        .text_size(px(12.))
        .text_color(rgb(t.error_fg))
        .child(message.clone())
        .into_any_element()
}
