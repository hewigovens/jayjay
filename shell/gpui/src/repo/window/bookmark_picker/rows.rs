use gpui::{
    AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};
use jayjay_core::BookmarkInfo;

use super::BookmarkPickerState;
use super::entry::BookmarkPickerEntry;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::repo::window::picker::{PickerSection, row, sections_by_best_match};
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::{self, glyph};

pub(super) fn bookmark_sections(
    state: &BookmarkPickerState,
    bookmarks: &[BookmarkInfo],
) -> Vec<PickerSection<BookmarkPickerEntry>> {
    let (mut tracked, mut local): (Vec<_>, Vec<_>) = bookmarks
        .iter()
        .filter(|bookmark| !bookmark.is_deleted && bookmark.has_local_target)
        .map(|bookmark| BookmarkPickerEntry {
            bookmark: bookmark.clone(),
            remote: None,
        })
        .partition(|entry| entry.bookmark.is_tracking_remote);
    let mut remote: Vec<_> = bookmarks
        .iter()
        .filter(|bookmark| !bookmark.has_local_target)
        .flat_map(|bookmark| {
            bookmark
                .available_remotes
                .iter()
                .filter(|remote| !bookmark.tracked_remotes.contains(remote))
                .map(|remote| BookmarkPickerEntry {
                    bookmark: bookmark.clone(),
                    remote: Some(remote.clone()),
                })
        })
        .collect();
    for entries in [&mut tracked, &mut local, &mut remote] {
        entries.sort_by_cached_key(|entry| entry.label().to_lowercase());
    }
    sections_by_best_match(
        [
            ("bookmark-picker-tracked", "Tracked", tracked),
            ("bookmark-picker-local", "Local Only", local),
            ("bookmark-picker-remote", "Remote Only", remote),
        ]
        .into_iter()
        .filter_map(|(id, title, rows)| {
            PickerSection::filtered(id, Some(title), rows, state.query.input.text(), |entry| {
                let bookmark = &entry.bookmark;
                if entry.remote.is_some() {
                    return entry.label();
                }
                format!(
                    "{} {} {}",
                    bookmark.name,
                    bookmark.tracked_remotes.join(" "),
                    bookmark.available_remotes.join(" ")
                )
            })
        }),
    )
}

pub(super) fn bookmark_row(
    entry: BookmarkPickerEntry,
    selected: bool,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let bookmark = &entry.bookmark;
    let caption = if entry.remote.is_some() {
        None
    } else {
        bookmark_caption(bookmark)
    };
    let height = if caption.is_some() { 38. } else { 28. };
    let label = entry.label();
    let revset = entry.revset();
    let context_revset = revset.clone();
    let click_view = view.clone();
    let context_view = view.clone();
    let context_bookmark = bookmark.clone();
    let remote = entry.remote.clone();
    row(entry.id(), selected, height, t)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            click_view.update(cx, |view, cx| view.filter_bookmark_revset(&revset, cx));
        })
        .on_mouse_down(MouseButton::Right, move |event: &MouseDownEvent, _, cx| {
            let anchor = event.position;
            let bookmark = context_bookmark.clone();
            context_view.update(cx, |view, cx| {
                let mut items = vec![ContextMenuItem::new(
                    "Filter by this bookmark",
                    glyph::FILTER,
                    ContextAction::FilterBookmarkRevset(context_revset.clone().into()),
                )];
                if let Some(remote) = &remote {
                    items.push(ContextMenuItem::new(
                        format!("Track {}@{remote}", bookmark.name),
                        glyph::GIT_BRANCH,
                        ContextAction::TrackBookmark {
                            name: bookmark.name.clone(),
                            remote: remote.clone(),
                        },
                    ));
                } else {
                    items.extend(view.build_bookmark_menu(&bookmark.name, None, cx));
                }
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
                                .child(SharedString::from(label)),
                        )
                        .child(if bookmark.is_tracking_remote || entry.remote.is_some() {
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
        assert_eq!(sections[0].rows[0].bookmark.name, "tracked");
        assert_eq!(sections[1].title, Some("Local Only"));
        assert_eq!(sections[1].rows[0].bookmark.name, "local");
        assert_eq!(
            picker_actions(&sections),
            vec![("\"tracked\"".to_owned(), 1), ("\"local\"".to_owned(), 3)]
        );
    }

    #[test]
    fn remote_rows_browse_each_remote_and_filter_by_qualified_name() {
        let mut state = BookmarkPickerState {
            anchor: gpui::point(gpui::px(0.), gpui::px(0.)),
            query: PickerQuery::new(),
        };
        let mut remote = bookmark("odd&name", false);
        remote.has_local_target = false;
        remote.available_remotes = vec!["upstream".to_owned(), "origin".to_owned()];
        let bookmarks = [bookmark("local", false), remote];
        let sections = bookmark_sections(&state, &bookmarks);
        assert_eq!(sections[1].title, Some("Remote Only"));
        assert_eq!(
            sections[1]
                .rows
                .iter()
                .map(|row| row.label())
                .collect::<Vec<_>>(),
            ["odd&name@origin", "odd&name@upstream"]
        );
        state.query.input.set_text("upstream".to_owned());
        let sections = bookmark_sections(&state, &bookmarks);
        assert_eq!(
            picker_actions(&sections),
            [(
                "ancestors(remote_bookmarks(exact:\"odd&name\", exact:\"upstream\"), 20)"
                    .to_owned(),
                1
            )]
        );
    }

    #[test]
    fn deleted_bookmark_keeps_its_untracked_remote() {
        let state = BookmarkPickerState {
            anchor: gpui::point(gpui::px(0.), gpui::px(0.)),
            query: PickerQuery::new(),
        };
        let mut remote = bookmark("feature", true);
        remote.has_local_target = false;
        remote.is_deleted = true;
        remote.available_remotes = vec!["origin".to_owned(), "upstream".to_owned()];
        let sections = bookmark_sections(&state, &[remote.clone()]);
        let labels: Vec<_> = sections
            .iter()
            .flat_map(|section| &section.rows)
            .map(|row| row.label())
            .collect();
        assert_eq!(labels, ["feature@upstream"]);

        remote.tracked_remotes.push("upstream".to_owned());
        assert!(bookmark_sections(&state, &[remote]).is_empty());
    }

    #[test]
    fn remote_row_identity_does_not_depend_on_its_display_label() {
        let first = super::BookmarkPickerEntry {
            bookmark: bookmark("a@b", false),
            remote: Some("c".to_owned()),
        };
        let second = super::BookmarkPickerEntry {
            bookmark: bookmark("a", false),
            remote: Some("b@c".to_owned()),
        };
        assert_eq!(first.label(), second.label());
        assert_ne!(first.id(), second.id());
        assert_ne!(first.revset(), second.revset());
    }
}
