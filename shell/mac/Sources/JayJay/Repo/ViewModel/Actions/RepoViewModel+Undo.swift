import Foundation
import JayJayCore

extension RepoViewModel {
    /// What Edit › Undo and Redo would step, or `nil` while an action or refresh is still landing.
    var operationSteps: UndoState? {
        pendingMutations == 0 && !isRefreshingInFlight ? undoState : nil
    }

    func undo(operationId: String) {
        step(from: operationId, message: Localized.Repo.undidOperation) { try $0.undo(operationId: operationId) }
    }

    func redo(operationId: String) {
        step(from: operationId, message: Localized.Repo.redidOperation) { try $0.redo(operationId: operationId) }
    }

    private func step(
        from operationId: String,
        message: @escaping (String) -> String,
        _ action: @escaping RepoOperation<String?>
    ) {
        performResult(
            onSuccess: { viewModel, description in
                if let description {
                    viewModel.info = message(description)
                } else {
                    viewModel.operationLogRequest = operationId
                }
            },
            action
        )
    }
}
