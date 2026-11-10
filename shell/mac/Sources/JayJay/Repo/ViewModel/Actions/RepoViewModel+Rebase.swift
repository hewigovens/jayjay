import Foundation
import JayJayCore

private struct RepoRebaseRefreshResult {
    let graph: GraphWithLayout
    let selectedChange: ChangeDetail?
    let workingCopyChangeId: String
    let workingCopyDescription: String
    let moved: Bool
    let hadConflicts: Bool
    let operationId: String?
    let context: RepoRefreshContext
}

extension RepoViewModel {
    func rebase(
        request: DAGRebaseRequest,
        onFailure: @escaping @MainActor (RepoViewModel, String) -> Void = { viewModel, message in
            viewModel.error = message
        }
    ) {
        // The confirmation can stay open across a refresh; never rebase commits it did not show.
        let shown = Set(graphEntries.map(\.change.commitId.id))
        guard ([request.sourceCommitId, request.destCommitId] + request.selectionCommitIds).allSatisfy(shown.contains) else {
            MainActor.assumeIsolated { onFailure(self, Localized.Repo.rebaseCancelledTheChangesMovedWhile) }
            return
        }
        rebase(request.command, onFailure: onFailure)
    }

    func rebase(
        _ command: RepoRebaseCommand,
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
                command: command,
                revset: requestedRevset,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
        } onSuccess: { viewModel, result in
            viewModel.successActionSignal += 1
            viewModel.setGraph(result.graph.entries, graph: result.graph)
            viewModel.applySingleSelectedChange(result.selectedChange)
            viewModel.applyWorkingCopy(
                changeId: result.workingCopyChangeId,
                description: result.workingCopyDescription
            )
            viewModel.apply(result.context)
            viewModel.isLoading = false
            viewModel.isRefreshingInFlight = false
            viewModel.canLoadMore = Self.canLoadMore(
                revset: viewModel.revset,
                loadedCount: result.graph.entries.count
            )
            viewModel.fetchPrInfo(bookmarks: result.selectedChange?.info.bookmarks ?? [])
            viewModel.resumePendingBackgroundRefresh()

            viewModel.actionNotice = RepoActionNotice(
                message: command.feedbackMessage(moved: result.moved, hadConflicts: result.hadConflicts),
                undoOperationId: result.moved ? result.operationId : nil
            )
        } onFailure: { viewModel, error in
            viewModel.isLoading = false
            viewModel.isRefreshingInFlight = false
            viewModel.resumePendingBackgroundRefresh(afterFailure: true)
            onFailure(viewModel, error.friendlyDescription)
        }
    }

    private static func rebaseAndReload(
        repo: JayJayRepo,
        command: RepoRebaseCommand,
        revset: String,
        includeSubmoduleStatuses: Bool
    ) throws -> RepoRebaseRefreshResult {
        let tracked = try trackingOperation(repo: repo) { repo -> (preferredRev: String, rebasedRevset: String) in
            if command.selectionRevs.isEmpty {
                let rebased = try repo.rebase(rev: command.sourceRev, dest: command.destRev, mode: command.mode)
                return (rebased, command.mode == .branch ? "(\(command.destRev))..\(rebased)" : rebased)
            }
            let source = try repo.showSummary(rev: command.sourceRev).info
            let rebasedChangeIds = try command.selectionRevs.map { try repo.showSummary(rev: $0).info.changeId.id }
            try repo.rebaseMany(revs: command.selectionRevs, dest: command.destRev)
            return (source.changeId.id, rebasedChangeIds.joined(separator: " | "))
        }
        let (preferredRev, rebasedRevset) = tracked.result
        let operationId = tracked.operationId
        // A rebase of changes already in place records no operation.
        let moved = operationId != nil
        try repo.refreshWorkingCopy()

        let graph = try repo.logGraphWithLayout(revset: revset)
        let log = graph.entries.map(\.change)
        let selectedChange = try loadSelectedDetail(
            repo: repo,
            log: log,
            preferredRev: preferredRev,
            includeSubmoduleStatuses: includeSubmoduleStatuses
        )
        let workingCopy = log.first(where: { $0.isWorkingCopy })
        let hadConflicts = try moved && repo.log(revset: rebasedRevset).contains(where: \.hasConflict)

        return try RepoRebaseRefreshResult(
            graph: graph,
            selectedChange: selectedChange,
            workingCopyChangeId: workingCopy?.changeId.id ?? "",
            workingCopyDescription: workingCopy?.description ?? "",
            moved: moved,
            hadConflicts: hadConflicts,
            operationId: operationId,
            context: RepoRefreshContext(repo: repo)
        )
    }
}
