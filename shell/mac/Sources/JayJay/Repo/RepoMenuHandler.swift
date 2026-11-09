import JayJayCore
import SwiftUI

@MainActor
final class RepoMenuHandler: RepositoryMenuHandler {
    func performShortcut(_ shortcut: AppShortcut) {
        onAction?(.shortcut(shortcut))
    }

    var onAction: ((MenuAction) -> Void)?
    weak var viewModel: RepoViewModel?

    enum MenuAction {
        case shortcut(AppShortcut)
        case commandPalette, operationLog, revsetFilter, bookmarkManager, overview, newWorkspace, pullRequestImport
    }

    func showCommandPalette() {
        onAction?(.commandPalette)
    }

    func showOperationLog() {
        onAction?(.operationLog)
    }

    func showRevsetFilter() {
        onAction?(.revsetFilter)
    }

    func showBookmarkManager() {
        onAction?(.bookmarkManager)
    }

    func showOverview() {
        onAction?(.overview)
    }

    func showNewWorkspace() {
        onAction?(.newWorkspace)
    }

    func showPullRequestImport() {
        onAction?(.pullRequestImport)
    }

    var operationSteps: UndoState? {
        viewModel?.operationSteps
    }

    func undoOperation() {
        guard let steps = operationSteps, steps.undoDescription != nil else { return }
        viewModel?.undo(operationId: steps.operationId)
    }

    func redoOperation() {
        guard let steps = operationSteps, steps.redoDescription != nil else { return }
        viewModel?.redo(operationId: steps.operationId)
    }
}
