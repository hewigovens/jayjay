import Foundation
import JayJayCore

extension RepoViewModel {
    func createBookmark(name: String, rev: String = "@") {
        perform(selecting: nil) { try $0.createBookmark(name: name, rev: rev) }
    }

    func moveBookmarkForward(name: String) {
        perform(selecting: nil) { try $0.moveBookmark(name: name, toRev: "@-") }
    }

    /// Move a bookmark to an arbitrary revision (drag-to-move). Backward moves are
    /// allowed silently, matching jj's default. Undoable via the op log. If the
    /// bookmark tracks a remote, offer a one-click push follow-up — but only once
    /// the move has actually succeeded (so the banner can't push the old target).
    func moveBookmark(name: String, toRev: String) {
        let wasTracking = bookmarks.first(where: { $0.name == name })?.isTrackingRemote == true
        perform(
            selecting: nil,
            beforeRefresh: { viewModel in
                if wasTracking { viewModel.pendingPushBookmark = name }
            },
            { try $0.moveBookmark(name: name, toRev: toRev) }
        )
    }

    func confirmPendingPush() {
        guard let bookmark = pendingPushBookmark else { return }
        if gitPushIfIdle(bookmark: bookmark) {
            pendingPushBookmark = nil
        }
    }

    func dismissPendingPush() {
        pendingPushBookmark = nil
    }

    func deleteBookmark(name: String) {
        perform(selecting: nil) { try $0.deleteBookmark(name: name) }
    }

    func forgetBookmark(name: String) {
        perform(selecting: nil) { try $0.forgetBookmark(name: name) }
    }

    func renameBookmark(oldName: String, newName: String) {
        perform(selecting: nil) { try $0.renameBookmark(oldName: oldName, newName: newName) }
    }

    func trackBookmark(name: String, remote: String) {
        perform(selecting: nil) { try $0.trackBookmark(name: name, remote: remote) }
    }
}
