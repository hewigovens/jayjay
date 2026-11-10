use crate::types::{BookmarkInfo, ChangeInfo, RemoteBookmarkTarget, RemoteSyncStatus};

/// A bookmark's state against its primary remote, VSCode-style: `name ✓` synced, `↑N` ahead, `↓N` behind, `↑N↓M` diverged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkSync {
    pub text: String,
    pub tooltip: String,
}

impl BookmarkSync {
    /// The nearest bookmark at or below `@` in graph order that has a remote.
    pub fn nearest_working_copy(
        changes: &[ChangeInfo],
        bookmarks: &[BookmarkInfo],
    ) -> Option<Self> {
        let wc_index = changes.iter().position(|change| change.is_working_copy)?;
        changes[wc_index..]
            .iter()
            .flat_map(|change| &change.bookmarks)
            .find_map(|name| {
                bookmarks
                    .iter()
                    .find(|bookmark| bookmark.name == *name)
                    .and_then(Self::for_bookmark)
            })
    }

    pub fn for_bookmark(bookmark: &BookmarkInfo) -> Option<Self> {
        let target = bookmark
            .remote_targets
            .iter()
            .find(|target| target.remote == "origin")
            .or_else(|| bookmark.remote_targets.first())?;
        Some(Self {
            text: format!("{} {}", bookmark.name, Self::badge(target)),
            tooltip: jayjay_i18n::repo::bookmark_vs_remote(&bookmark.name, &target.remote),
        })
    }

    fn badge(target: &RemoteBookmarkTarget) -> String {
        match target.status {
            RemoteSyncStatus::Deleted => jayjay_i18n::repo::deleted_on(&target.remote),
            RemoteSyncStatus::Synced => "✓".to_owned(),
            RemoteSyncStatus::Ahead => format!("↑{}", target.ahead),
            RemoteSyncStatus::Behind => format!("↓{}", target.behind),
            RemoteSyncStatus::Diverged => format!("↑{}↓{}", target.ahead, target.behind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{bookmark_info, change_info};

    fn tracked(name: &str, remotes: &[(&str, RemoteSyncStatus)]) -> BookmarkInfo {
        BookmarkInfo {
            remote_targets: remotes
                .iter()
                .map(|&(remote, status)| RemoteBookmarkTarget {
                    remote: remote.to_owned(),
                    change_id: String::new(),
                    description: String::new(),
                    status,
                    ahead: 2,
                    behind: 3,
                })
                .collect(),
            ..bookmark_info(name)
        }
    }

    #[test]
    fn badge_prefers_origin_and_formats_each_status() {
        for (status, badge) in [
            (RemoteSyncStatus::Synced, "✓"),
            (RemoteSyncStatus::Ahead, "↑2"),
            (RemoteSyncStatus::Behind, "↓3"),
            (RemoteSyncStatus::Diverged, "↑2↓3"),
            (RemoteSyncStatus::Deleted, "deleted on origin"),
        ] {
            let bookmark = tracked(
                "main",
                &[("upstream", RemoteSyncStatus::Synced), ("origin", status)],
            );
            assert_eq!(
                BookmarkSync::for_bookmark(&bookmark),
                Some(BookmarkSync {
                    text: format!("main {badge}"),
                    tooltip: "main vs origin".to_owned(),
                })
            );
        }
        let upstream = tracked("main", &[("upstream", RemoteSyncStatus::Synced)]);
        assert_eq!(
            BookmarkSync::for_bookmark(&upstream).map(|sync| sync.tooltip),
            Some("main vs upstream".to_owned())
        );
        assert_eq!(BookmarkSync::for_bookmark(&bookmark_info("local")), None);
    }

    #[test]
    fn nearest_working_copy_skips_rows_above_at_and_local_only_bookmarks() {
        let mut above = change_info("above", "a");
        above.bookmarks = vec!["above".to_owned()];
        let mut wc = change_info("wc", "w");
        wc.is_working_copy = true;
        wc.bookmarks = vec!["local".to_owned()];
        let mut parent = change_info("parent", "p");
        parent.bookmarks = vec!["feature".to_owned()];
        let bookmarks = [
            tracked("above", &[("origin", RemoteSyncStatus::Synced)]),
            bookmark_info("local"),
            tracked("feature", &[("origin", RemoteSyncStatus::Ahead)]),
        ];

        let sync = BookmarkSync::nearest_working_copy(&[above, wc, parent], &bookmarks);

        assert_eq!(sync.map(|sync| sync.text), Some("feature ↑2".to_owned()));
    }
}
