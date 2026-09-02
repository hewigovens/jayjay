import Foundation
import JayJayCore

struct RepoRebaseFeedback {
    let message: String
    let undoOperationId: String?
}

private struct RepoRebaseRefreshResult {
    let graphEntries: [GraphEntry]
    let dagLayout: DAGLayout
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]?
    let selectedChange: ChangeDetail?
    let workingCopyChangeId: String
    let workingCopyDescription: String
    let hadConflicts: Bool
    let undoOperationId: String?
    let statusBar: StatusBarSnapshot
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
            try Self.rebaseAndReload(
                repo: repo,
                request: request,
                revset: requestedRevset,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
        } onSuccess: { viewModel, result in
            viewModel.successActionSignal += 1
            viewModel.graphEntries = result.graphEntries
            viewModel.dagLayout = result.dagLayout
            viewModel.bookmarks = result.bookmarks
            if let workspaces = result.workspaces {
                viewModel.workspaces = workspaces
            }
            viewModel.applySingleSelectedChange(result.selectedChange)
            viewModel.applyWorkingCopy(
                changeId: result.workingCopyChangeId,
                description: result.workingCopyDescription
            )
            viewModel.apply(result.statusBar)
            viewModel.isLoading = false
            viewModel.isRefreshingInFlight = false
            viewModel.canLoadMore = Self.canLoadMore(
                revset: viewModel.revset,
                loadedCount: result.graphEntries.count
            )
            viewModel.fetchPrInfo(bookmarks: result.selectedChange?.info.bookmarks ?? [])
            viewModel.resumePendingBackgroundRefresh()

            onSuccess(viewModel, RepoRebaseFeedback(
                message: Self.rebaseMessage(for: request, hadConflicts: result.hadConflicts),
                undoOperationId: result.undoOperationId
            ))
        } onFailure: { viewModel, error in
            viewModel.isLoading = false
            viewModel.isRefreshingInFlight = false
            viewModel.resumePendingBackgroundRefresh()
            onFailure(viewModel, error.friendlyDescription)
        }
    }

    private static func rebaseAndReload(
        repo: JayJayRepo,
        request: DAGRebaseRequest,
        revset: String,
        includeSubmoduleStatuses: Bool
    ) throws -> RepoRebaseRefreshResult {
        let undoOperationId = try repo.opLog().first(where: { $0.isCurrent })?.id.id
        try repo.rebase(rev: request.sourceRev, dest: request.destRev)
        try repo.refreshWorkingCopy()

        let graphEntries = try repo.logGraph(revset: revset)
        let dagLayout = DAGLayout(entries: graphEntries)
        let log = graphEntries.map(\.change)
        let bookmarks = try repo.listBookmarks()
        let workspaces = try? repo.workspaceList()
        let selectedChange = try loadSelectedDetail(
            repo: repo,
            log: log,
            preferredRev: request.sourceChangeId,
            includeSubmoduleStatuses: includeSubmoduleStatuses
        )
        let workingCopy = log.first(where: { $0.isWorkingCopy })
        let hadConflicts = graphEntries.contains(where: {
            $0.change.changeId.id == request.sourceChangeId && $0.change.hasConflict
        })

        return RepoRebaseRefreshResult(
            graphEntries: graphEntries,
            dagLayout: dagLayout,
            bookmarks: bookmarks,
            workspaces: workspaces,
            selectedChange: selectedChange,
            workingCopyChangeId: workingCopy?.changeId.id ?? "",
            workingCopyDescription: workingCopy?.description ?? "",
            hadConflicts: hadConflicts,
            undoOperationId: undoOperationId,
            statusBar: StatusBarSnapshot.load(from: repo)
        )
    }

    private static func rebaseMessage(for request: DAGRebaseRequest, hadConflicts: Bool) -> String {
        let base = "Rebased \(request.sourceLabel) onto \(request.destLabel)."
        guard hadConflicts else { return base }
        return "\(base) Conflicts need resolution."
    }
}
