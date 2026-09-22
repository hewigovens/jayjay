import SwiftUI

@MainActor
final class RepoMenuHandler: RepositoryMenuHandler {
    var onAction: ((MenuAction) -> Void)?

    enum MenuAction {
        case commandPalette, undo, bookmarkManager, newWorkspace, pullRequestImport
    }

    func showCommandPalette() {
        onAction?(.commandPalette)
    }

    func showUndo() {
        onAction?(.undo)
    }

    func showBookmarkManager() {
        onAction?(.bookmarkManager)
    }

    func showNewWorkspace() {
        onAction?(.newWorkspace)
    }

    func showPullRequestImport() {
        onAction?(.pullRequestImport)
    }
}
