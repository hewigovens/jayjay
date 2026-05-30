use std::sync::Arc;

use gpui::{
    AnyElement, ClipboardItem, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
    uniform_list,
};
use jayjay_core::BookmarkInfo;

use super::BookmarkManagerView;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::primitives::{capsule, no_scrollbar_gutter};
use crate::ui::text_area::button;

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
        .h_full()
        .w_full()
        .into_any_element()
}

fn bookmark_row(
    bookmark: BookmarkInfo,
    t: Arc<Theme>,
    cx: &mut Context<BookmarkManagerView>,
) -> AnyElement {
    let name = bookmark.name.clone();
    let change_id = bookmark.change_id.clone();
    let description = if bookmark.description.trim().is_empty() {
        "(no description)".to_owned()
    } else {
        bookmark.description.clone()
    };
    let remotes = remotes_label(&bookmark);
    let track_remote = (!bookmark.is_tracking_remote)
        .then(|| bookmark.available_remotes.first().cloned())
        .flatten();
    let bookmark_for_diff = bookmark.clone();
    let bookmark_for_menu = bookmark.clone();
    let name_for_copy = name.clone();
    let name_for_id = name_for_copy.replace('/', "-");
    let change_for_reveal = change_id.clone();
    let bookmark_for_track = bookmark.clone();
    let can_target = !change_id.is_empty();

    div()
        .id(SharedString::from(format!("bookmark-row-{name}")))
        .flex()
        .flex_row()
        .w_full()
        .items_center()
        .gap(px(12.))
        .px(px(16.))
        .py(px(10.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |view, ev: &MouseDownEvent, _window, cx| {
                view.open_context_menu(ev.position, bookmark_for_menu.clone(), cx);
            }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
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
                                .text_size(px(12.))
                                .text_color(rgb(t.fg))
                                .child(SharedString::from(name)),
                        )
                        .children(status_chips(&bookmark, &t)),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(description)),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_faint))
                        .child(SharedString::from(remotes)),
                ),
        )
        .children(can_target.then(|| {
            row_button(format!("reveal-{name_for_id}"), "Reveal", &t).on_click(cx.listener(
                move |view, _, _, cx| {
                    view.reveal(change_for_reveal.clone(), cx);
                },
            ))
        }))
        .children(can_target.then(|| {
            row_button(format!("diff-{name_for_id}"), "Diff", &t).on_click(cx.listener(
                move |view, _, _, cx| {
                    view.show_diff(bookmark_for_diff.clone(), cx);
                },
            ))
        }))
        .child(
            row_button(format!("copy-{name_for_id}"), "Copy", &t).on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(name_for_copy.clone()));
            }),
        )
        .children(track_remote.map(|remote| {
            let remote_for_click = remote.clone();
            row_button(format!("track-{name_for_id}"), "Track", &t)
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.track_bookmark(
                        bookmark_for_track.name.clone(),
                        remote_for_click.clone(),
                        cx,
                    );
                }))
                .into_any_element()
        }))
        .into_any_element()
}

fn row_button(id: String, label: &'static str, t: &Theme) -> gpui::Stateful<gpui::Div> {
    button(SharedString::from(id), label, t, false)
}

fn status_chips(bookmark: &BookmarkInfo, t: &Theme) -> Vec<AnyElement> {
    let mut chips = Vec::new();
    if bookmark.is_conflicted {
        chips.push(
            capsule("conflict", t.tag_conflict_bg, t.tag_conflict_fg, 10.).into_any_element(),
        );
    }
    if bookmark.is_deleted {
        chips.push(capsule("deleted", t.tag_removed_bg, t.tag_removed_fg, 10.).into_any_element());
    } else if !bookmark.has_local_target {
        chips.push(capsule("remote", t.tag_bg, t.tag_fg, 10.).into_any_element());
    } else if bookmark.is_tracking_remote {
        chips
            .push(capsule("tracked", t.tag_bookmark_bg, t.tag_bookmark_fg, 10.).into_any_element());
    } else {
        chips.push(capsule("local", t.tag_bg, t.tag_fg, 10.).into_any_element());
    }
    chips
}

fn remotes_label(bookmark: &BookmarkInfo) -> String {
    if bookmark.tracked_remotes.is_empty() && bookmark.available_remotes.is_empty() {
        return "No remote".to_owned();
    }
    if !bookmark.tracked_remotes.is_empty() {
        return format!("Tracking {}", bookmark.tracked_remotes.join(", "));
    }
    format!("Available on {}", bookmark.available_remotes.join(", "))
}
