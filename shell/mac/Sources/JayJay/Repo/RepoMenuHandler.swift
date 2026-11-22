import JayJayCore
import SwiftUI

@MainActor
@Observable
final class RepoMenuHandler: RepositoryMenuHandler {
    /// Pushed by the window whenever what its shortcuts would act on changes, so the Repository menu disables what would do nothing.
    var availableShortcuts: Set<AppShortcut> = []
    var canViewRemoteRepository = false
    var canRestoreOperation = false
    var reviewMode = ReviewMode.unavailable

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
        case commandPalette, revsetFilter, bookmarkManager, overview, newWorkspace, pullRequestImport, restoreOperation
    }

    func showCommandPalette() {
        onAction?(.commandPalette)
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

    func restoreOperation() {
        onAction?(.restoreOperation)
    }

    var operationSteps: UndoState? {
        viewModel?.operationSteps
    }

    func undoOperation() {
        guard let steps = operationSteps else { return }
        if steps.undoDescription == nil {
            viewModel?.explainUnavailableUndo()
        } else {
            viewModel?.undo(operationId: steps.operationId)
        }
    }

    func redoOperation() {
        guard let steps = operationSteps, steps.redoDescription != nil else { return }
        viewModel?.redo(operationId: steps.operationId)
    }
}
