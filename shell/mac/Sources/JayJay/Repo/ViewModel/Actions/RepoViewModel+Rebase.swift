import Foundation
import JayJayCore

struct RepoRebaseFeedback {
    let message: String
    let undoOperationId: String?
}

private struct RepoRebaseRefreshResult {
    let graphEntries: [GraphEntry]
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]
    let selectedChange: ChangeDetail?
    let workingCopyDescription: String
    let hadConflicts: Bool
    let undoOperationId: String?
}

extension RepoViewModel {
    func rebase(
        request: DAGRebaseRequest,
        onSuccess: @escaping @MainActor (RepoViewModel, RepoRebaseFeedback) -> Void,
        onFailure: @escaping @MainActor (RepoViewModel, String) -> Void = { viewModel, message in
            viewModel.error = message
        }
    ) {
        lastInternalMutationAt = Date()
        isRefreshingInFlight = true
        error = nil
        let includeSubmoduleStatuses = includeSubmoduleStatuses

        runRepoTask { [requestedRevset = revset, includeSubmoduleStatuses] repo in
            let undoOperationId = try repo.opLog().first(where: { $0.isCurrent })?.id
            try repo.rebase(rev: request.sourceRev, dest: request.destRev)
            try repo.refreshWorkingCopy()

            let graphEntries = try repo.logGraph(revset: requestedRevset)
            let log = graphEntries.map(\.change)
            let bookmarks = try repo.listBookmarks()
            let workspaces = (try? repo.workspaceList()) ?? []
            let selectedChange = try Self.loadSelectedDetail(
                repo: repo,
                log: log,
                preferredRev: request.sourceChangeId,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
            let workingCopyDescription = log.first(where: { $0.isWorkingCopy })?.description ?? ""
            let hadConflicts = graphEntries.contains(where: {
                $0.change.changeId.id == request.sourceChangeId && $0.change.hasConflict
            })

            return RepoRebaseRefreshResult(
                graphEntries: graphEntries,
                bookmarks: bookmarks,
                workspaces: workspaces,
                selectedChange: selectedChange,
                workingCopyDescription: workingCopyDescription,
                hadConflicts: hadConflicts,
                undoOperationId: undoOperationId
            )
        } onSuccess: { viewModel, result in
            viewModel.successActionSignal += 1
            viewModel.graphEntries = result.graphEntries
            viewModel.bookmarks = result.bookmarks
            viewModel.workspaces = result.workspaces
            viewModel.selectedChange = result.selectedChange
            viewModel.selectedChangeId = result.selectedChange?.info.selectionRevision
            viewModel.workingCopyDescription = result.workingCopyDescription
            viewModel.isLoading = false
            viewModel.isRefreshingInFlight = false
            viewModel.hasWorkingCopyChanges = false
            viewModel.canLoadMore = Self.canLoadMore(
                revset: viewModel.revset,
                loadedCount: result.graphEntries.count
            )
            viewModel.fetchPrInfo(bookmarks: result.selectedChange?.info.bookmarks ?? [])

            onSuccess(viewModel, RepoRebaseFeedback(
                message: Self.rebaseMessage(for: request, hadConflicts: result.hadConflicts),
                undoOperationId: result.undoOperationId
            ))
        } onFailure: { viewModel, error in
            viewModel.isLoading = false
            viewModel.isRefreshingInFlight = false
            onFailure(viewModel, error.friendlyDescription)
        }
    }

    private static func rebaseMessage(for request: DAGRebaseRequest, hadConflicts: Bool) -> String {
        let base = "Rebased \(request.sourceLabel) onto \(request.destLabel)."
        guard hadConflicts else { return base }
        return "\(base) Conflicts need resolution."
    }
}
