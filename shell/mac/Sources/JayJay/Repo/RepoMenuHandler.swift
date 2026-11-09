import JayJayCore
import SwiftUI

@MainActor
@Observable
final class RepoMenuHandler: RepositoryMenuHandler {
    /// Pushed by the window whenever what its shortcuts would act on changes, so the Repository menu disables what would do nothing.
    var availableShortcuts: Set<AppShortcut> = []
    var canViewRemoteRepository = false

    func canPerformShortcut(_ shortcut: AppShortcut) -> Bool {
        availableShortcuts.contains(shortcut)
    }

    func performShortcut(_ shortcut: AppShortcut) {
        onAction?(.shortcut(shortcut))
    }

    @ObservationIgnored var onAction: ((MenuAction) -> Void)?
    @ObservationIgnored weak var viewModel: RepoViewModel?

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
