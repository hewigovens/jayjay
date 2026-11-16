use gpui::{AnyElement, Entity};
use jayjay_core::{BookmarkInfo, trunk};

use super::BookmarkManagerView;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::popup_menu::{PopupMenu, PopupMenuEntry, render_popup_menu};

#[derive(Clone)]
pub(super) enum BookmarkContextAction {
    Reveal(String),
    ShowDiff(BookmarkInfo),
    Track { name: String, remote: String },
    Push(String),
    Resolve(String),
    OpenPullRequest(String),
    Rename(String),
    Delete(String),
    Forget(String),
}

pub(super) fn bookmark_menu_items(
    bookmark: &BookmarkInfo,
    pr_host_name: Option<&str>,
) -> Vec<PopupMenuEntry<BookmarkContextAction>> {
    let mut items = Vec::new();
    if !bookmark.is_deleted && !bookmark.change_id.is_empty() {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::bookmarks::reveal(),
            glyph::ARROW_CIRCLE_RIGHT,
            BookmarkContextAction::Reveal(bookmark.change_id.id.clone()),
        ));
    }
    if !bookmark.is_deleted
        && !bookmark.is_conflicted
        && !bookmark.change_id.is_empty()
        && !trunk::is_trunk_bookmark(&bookmark.name)
    {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::bookmarks::diff(),
            glyph::ARROWS_LEFT_RIGHT,
            BookmarkContextAction::ShowDiff(bookmark.clone()),
        ));
    }
    if bookmark.is_conflicted {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::menu::resolve_conflict_set_to(),
            glyph::GIT_MERGE,
            BookmarkContextAction::Resolve(bookmark.name.clone()),
        ));
    }
    if !bookmark.is_deleted && !bookmark.has_local_target {
        for remote in &bookmark.available_remotes {
            items.push(PopupMenuEntry::item(
                jayjay_i18n::repo::track_remote_bookmark(&bookmark.name, remote),
                glyph::GIT_BRANCH,
                BookmarkContextAction::Track {
                    name: bookmark.name.clone(),
                    remote: remote.clone(),
                },
            ));
        }
    } else if bookmark.is_tracking_remote && !bookmark.is_deleted {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::menu::push(),
            glyph::ARROW_UP,
            BookmarkContextAction::Push(bookmark.name.clone()),
        ));
    }
    if let Some(host_name) = pr_host_name
        && bookmark.is_tracking_remote
        && !bookmark.is_deleted
        && !trunk::is_trunk_bookmark(&bookmark.name)
    {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::repo::pull_request_on(host_name),
            glyph::EXTERNAL_LINK,
            BookmarkContextAction::OpenPullRequest(bookmark.name.clone()),
        ));
    }
    if bookmark.is_deleted {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::repo::forget_clean_up(),
            glyph::BOOKMARK,
            BookmarkContextAction::Forget(bookmark.name.clone()),
        ));
    } else if bookmark.has_local_target {
        items.push(PopupMenuEntry::item(
            jayjay_i18n::repo::rename(),
            glyph::PENCIL,
            BookmarkContextAction::Rename(bookmark.name.clone()),
        ));
        items.push(PopupMenuEntry::item(
            jayjay_i18n::repo::delete(),
            glyph::X_CIRCLE,
            BookmarkContextAction::Delete(bookmark.name.clone()),
        ));
    }
    items
}

pub(super) fn render_context_menu(
    menu: &PopupMenu<BookmarkContextAction>,
    t: &Theme,
    view: &Entity<BookmarkManagerView>,
) -> AnyElement {
    let dismiss = view.clone();
    let select = view.clone();
    render_popup_menu(
        menu,
        "bookmark-context",
        t,
        move |_, cx| dismiss.update(cx, |view, cx| view.close_context_menu(cx)),
        move |action, _, cx| select.update(cx, |view, cx| view.dispatch_context_action(action, cx)),
    )
}

#[cfg(test)]
mod tests {
    use jayjay_core::BookmarkInfo;
    use jayjay_core::mock::{bookmark_info, strings};

    use super::bookmark_menu_items;
    use crate::ui::popup_menu::PopupMenuEntry;

    fn bookmark(name: &str) -> BookmarkInfo {
        BookmarkInfo {
            is_tracking_remote: true,
            tracked_remotes: strings(&["origin"]),
            available_remotes: strings(&["origin"]),
            ..bookmark_info(name)
        }
    }

    fn labels(bookmark: &BookmarkInfo, pr_host_name: Option<&str>) -> Vec<String> {
        bookmark_menu_items(bookmark, pr_host_name)
            .into_iter()
            .filter_map(|entry| match entry {
                PopupMenuEntry::Item { label, .. } => Some(label.to_string()),
                PopupMenuEntry::Separator => None,
            })
            .collect()
    }

    #[test]
    fn deleted_bookmark_offers_only_cleanup() {
        let mut bookmark = bookmark("stale");
        bookmark.is_deleted = true;

        assert_eq!(labels(&bookmark, Some("GitHub")), ["Forget (clean up)"]);
    }

    #[test]
    fn tracked_bookmark_matches_swiftui_actions() {
        assert_eq!(
            labels(&bookmark("feature"), Some("GitHub")),
            [
                "Reveal",
                "Diff",
                "Push",
                "Pull Request on GitHub",
                "Rename",
                "Delete",
            ]
        );
    }

    #[test]
    fn unknown_host_keeps_git_actions_without_pull_request() {
        assert_eq!(
            labels(&bookmark("feature"), None),
            ["Reveal", "Diff", "Push", "Rename", "Delete"]
        );
    }
}
