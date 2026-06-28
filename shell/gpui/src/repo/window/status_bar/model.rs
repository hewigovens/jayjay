use jayjay_core::{BookmarkInfo, ChangeInfo, DiffStats, RemoteBookmarkTarget, RemoteSyncStatus};

pub(super) fn active_bookmark_sync_label(
    changes: &[ChangeInfo],
    bookmarks: &[BookmarkInfo],
) -> Option<String> {
    let wc_index = changes.iter().position(|change| change.is_working_copy)?;
    for change in &changes[wc_index..] {
        for name in &change.bookmarks {
            let Some(bookmark) = bookmarks.iter().find(|bookmark| bookmark.name == *name) else {
                continue;
            };
            let Some(target) = primary_remote_target(bookmark) else {
                continue;
            };
            let badge = sync_badge(target);
            return Some(if badge.is_empty() {
                name.clone()
            } else {
                format!("{name} {badge}")
            });
        }
    }
    None
}

pub(super) fn working_copy_stat_label(stats: &DiffStats) -> Option<String> {
    if stats.files_changed == 0 {
        return None;
    }
    let plural = if stats.files_changed == 1 { "" } else { "s" };
    let mut text = format!("{} file{plural}", stats.files_changed);
    if stats.insertions > 0 || stats.deletions > 0 {
        text.push_str(&format!(" +{} -{}", stats.insertions, stats.deletions));
    }
    Some(text)
}

fn primary_remote_target(bookmark: &BookmarkInfo) -> Option<&RemoteBookmarkTarget> {
    bookmark
        .remote_targets
        .iter()
        .find(|target| target.remote == "origin")
        .or_else(|| bookmark.remote_targets.first())
}

fn sync_badge(target: &RemoteBookmarkTarget) -> String {
    match target.status {
        RemoteSyncStatus::Synced => "✓".to_owned(),
        RemoteSyncStatus::Ahead => format!("↑{}", target.ahead),
        RemoteSyncStatus::Behind => format!("↓{}", target.behind),
        RemoteSyncStatus::Diverged => format!("↑{}↓{}", target.ahead, target.behind),
    }
}
