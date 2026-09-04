use gpui::{
    AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};
use jayjay_core::BookmarkInfo;

use super::BookmarkPickerState;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::repo::window::picker::{PickerRow, PickerSection, row, sections_by_best_match};
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::{self, glyph};

impl PickerRow for BookmarkInfo {
    type Action = String;

    fn action(&self) -> Option<String> {
        Some(self.name.clone())
    }
}

pub(super) fn bookmark_sections(
    state: &BookmarkPickerState,
    bookmarks: &[BookmarkInfo],
) -> Vec<PickerSection<BookmarkInfo>> {
    let (mut tracked, mut local): (Vec<_>, Vec<_>) = bookmarks
        .iter()
        .filter(|bookmark| !bookmark.is_deleted && bookmark.has_local_target)
        .cloned()
        .partition(|bookmark| bookmark.is_tracking_remote);
    tracked.sort_by_cached_key(|bookmark| bookmark.name.to_lowercase());
    local.sort_by_cached_key(|bookmark| bookmark.name.to_lowercase());
    sections_by_best_match(
        [
            ("bookmark-picker-tracked", "Tracked", tracked),
            ("bookmark-picker-local", "Local Only", local),
        ]
        .into_iter()
        .filter_map(|(id, title, rows)| {
            PickerSection::filtered(
                id,
                Some(title),
                rows,
                state.query.input.text(),
                |bookmark| {
                    format!(
                        "{} {} {}",
                        bookmark.name,
                        bookmark.tracked_remotes.join(" "),
                        bookmark.available_remotes.join(" ")
                    )
                },
            )
        }),
    )
}

pub(super) fn bookmark_row(
    bookmark: BookmarkInfo,
    selected: bool,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let caption = bookmark_caption(&bookmark);
    let height = if caption.is_some() { 38. } else { 28. };
    let id = format!("bookmark-picker-row-{}", bookmark.name);
    let name = bookmark.name.clone();
    let click_view = view.clone();
    let context_view = view.clone();
    let context_bookmark = bookmark.clone();
    row(id, selected, height, t)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            click_view.update(cx, |view, cx| view.filter_by_bookmark(&name, cx));
        })
        .on_mouse_down(MouseButton::Right, move |event: &MouseDownEvent, _, cx| {
            let anchor = event.position;
            let bookmark = context_bookmark.clone();
            context_view.update(cx, |view, cx| {
                let mut items = vec![ContextMenuItem::new(
                    "Filter by this bookmark",
                    glyph::FILTER,
                    ContextAction::FilterByBookmark(bookmark.name.clone().into()),
                )];
                items.extend(view.build_bookmark_menu(&bookmark.name, None, cx));
                view.open_context_menu(anchor, items, cx);
            });
        })
        .child(
            div()
                .flex()
                .min_w_0()
                .flex_1()
                .flex_col()
                .gap(px(1.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .min_w_0()
                        .gap(px(6.))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_size(ui_font_size(13.))
                                .child(SharedString::from(bookmark.name)),
                        )
                        .child(if bookmark.is_tracking_remote {
                            icons::icon(glyph::CLOUD, 11., t.fg_dim)
                        } else {
                            icons::icon(glyph::CLOUD_OFF, 11., t.fg_faint)
                        }),
                )
                .children(caption.map(|caption| {
                    div()
                        .truncate()
                        .text_size(ui_font_size(10.))
                        .text_color(rgb(t.fg_dim))
                        .child(caption)
                })),
        )
        .into_any_element()
}

fn bookmark_caption(bookmark: &BookmarkInfo) -> Option<String> {
    if !bookmark.tracked_remotes.is_empty() {
        return Some(
            bookmark
                .tracked_remotes
                .iter()
                .map(|remote| format!("@{remote}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    (!bookmark.available_remotes.is_empty()).then(|| {
        format!(
            "Remote available: {}",
            bookmark
                .available_remotes
                .iter()
                .map(|remote| format!("@{remote}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

#[cfg(test)]
mod tests {
    use jayjay_core::{BookmarkInfo, ShortId};

    use super::{BookmarkPickerState, bookmark_sections};
    use crate::repo::window::picker::{PickerQuery, picker_actions};

    fn bookmark(name: &str, tracking: bool) -> BookmarkInfo {
        BookmarkInfo {
            name: name.to_owned(),
            change_id: ShortId::new("abcdefghijkl".to_owned(), 3),
            description: String::new(),
            is_tracking_remote: tracking,
            is_deleted: false,
            is_conflicted: false,
            tracked_remotes: tracking.then(|| "origin".to_owned()).into_iter().collect(),
            available_remotes: Vec::new(),
            has_local_target: true,
            remote_targets: Vec::new(),
        }
    }

    #[test]
    fn sections_keep_tracked_and_local_only_bookmarks_separate() {
        let state = BookmarkPickerState {
            anchor: gpui::point(gpui::px(0.), gpui::px(0.)),
            query: PickerQuery::new(),
        };

        let sections = bookmark_sections(
            &state,
            &[bookmark("local", false), bookmark("tracked", true)],
        );

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, Some("Tracked"));
        assert_eq!(sections[0].rows[0].name, "tracked");
        assert_eq!(sections[1].title, Some("Local Only"));
        assert_eq!(sections[1].rows[0].name, "local");
        assert_eq!(
            picker_actions(&sections),
            vec![("tracked".to_owned(), 1), ("local".to_owned(), 3)]
        );
    }
}
