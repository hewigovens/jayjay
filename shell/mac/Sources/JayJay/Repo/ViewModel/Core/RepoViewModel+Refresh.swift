import Foundation
import JayJayCore

private struct RepoRefreshContent {
    let graph: GraphWithLayout
    let selectedChange: ChangeDetail?
    let workingCopyChangeId: String
    let workingCopyDescription: String
    let context: RepoRefreshContext?
}

enum BackgroundRefreshRequest {
    case checkOperation
    case reload
}

extension RepoViewModel {
    func handleOperationChange() {
        pendingBackgroundRefresh = pendingBackgroundRefresh ?? .checkOperation
        resumePendingBackgroundRefresh()
    }

    func handleWorkingCopyChange() {
        guard !isShuttingDown else { return }
        // Editing defers events even within the mutation echo window.
        if !isBackgroundRefreshSuspended,
           let last = lastInternalMutationAt, Date().timeIntervalSince(last) < 5
        {
            return
        }
        pendingBackgroundRefresh = .reload
        resumePendingBackgroundRefresh()
    }

    func setBackgroundRefreshSuspended(_ suspended: Bool) {
        guard isBackgroundRefreshSuspended != suspended else { return }
        isBackgroundRefreshSuspended = suspended
        resumePendingBackgroundRefresh()
    }

    func fetchPrInfo(bookmarks: [String]) {
        clearPrInfo()
        guard !isShuttingDown, let bookmark = bookmarks.first else { return }
        prFetchTask = startRepoTask { [weak self, repo] in
            let info = repo.pullRequestInfo(bookmark: bookmark)
            guard !Task.isCancelled else { return }
            await self?.applyPrInfo(info)
        }
    }

    func clearPrInfo() {
        prFetchTask?.cancel()
        prFetchTask = nil
        prInfo = nil
    }

    @MainActor
    private func applyPrInfo(_ info: PrInfo?) {
        guard !isShuttingDown else { return }
        prInfo = info
    }

    func applyRevset(_ newRevset: String, selecting revision: String = "@") {
        revset = newRevset
        canLoadMore = Self.canLoadMore(revset: newRevset, loadedCount: graphEntries.count)
        refresh(selecting: revision)
    }

    func refresh(
        selecting preferredRev: String? = nil,
        isAutoTriggered: Bool = false,
        snapshotWorkingCopy: Bool = true
    ) {
        guard !isShuttingDown else { return }
        if isAutoTriggered, isRefreshingInFlight {
            pendingBackgroundRefresh = .reload
            return
        }
        refreshTask?.cancel()
        isRefreshingInFlight = true
        isLoading = graphEntries.isEmpty
        // A background refresh must not dismiss an error the user is still reading; manual refresh is an explicit retry.
        if !isAutoTriggered {
            error = nil
        }
        let currentSelection = selectedChangeId
        let requestedRevset = revset
        let includeSubmoduleStatuses = includeSubmoduleStatuses
        let shouldLoadBeforeSnapshot = graphEntries.isEmpty && snapshotWorkingCopy
        // A requested revision wins the first pass; later passes preserve subsequent user selections.
        var selectionBaseline = preferredRev == nil ? selectedChangeIds : nil
        let repo = repo
        let load = { includeContext in
            try Self.loadRefreshContent(
                repo: repo,
                revset: requestedRevset,
                preferredRev: preferredRev ?? currentSelection,
                includeSubmoduleStatuses: includeSubmoduleStatuses,
                includeContext: includeContext
            )
        }
        refreshTask = startRepoTask { [weak self, repo] in
            do {
                if shouldLoadBeforeSnapshot {
                    let content = try load(false)
                    guard !Task.isCancelled else { return }
                    selectionBaseline = await self?.applyRefreshContent(
                        content,
                        revset: requestedRevset,
                        isAutoTriggered: isAutoTriggered,
                        selectionBaseline: selectionBaseline
                    )
                }

                if snapshotWorkingCopy {
                    try repo.refreshWorkingCopy()
                    guard !Task.isCancelled else { return }
                }

                let content = try load(true)
                guard !Task.isCancelled else { return }
                await self?.applyRefreshContent(
                    content,
                    revset: requestedRevset,
                    isAutoTriggered: isAutoTriggered,
                    selectionBaseline: selectionBaseline
                )
            } catch {
                guard !Task.isCancelled else { return }
                // A failed snapshot must still populate the context omitted from the first paint.
                let context = shouldLoadBeforeSnapshot ? try? RepoRefreshContext(repo: repo) : nil
                await self?.applyRefreshFailure(error, presence: repo.workspacePresence(), context: context)
            }
        }
    }

    @MainActor
    @discardableResult
    private func applyRefreshContent(
        _ content: RepoRefreshContent,
        revset: String,
        isAutoTriggered: Bool,
        selectionBaseline: [String]?
    ) -> [String]? {
        guard !Task.isCancelled, !isShuttingDown else { return selectionBaseline }
        let isRefreshComplete = content.context != nil
        if isAutoTriggered, isBackgroundRefreshSuspended {
            pendingBackgroundRefresh = .reload
            if isRefreshComplete {
                isRefreshingInFlight = false
            }
            return selectionBaseline
        }
        let selectsLoadedChange = selectionBaseline == nil || selectionBaseline == selectedChangeIds
        apply(content, selectsLoadedChange: selectsLoadedChange)
        canLoadMore = Self.canLoadMore(
            revset: revset,
            loadedCount: content.graph.entries.count
        )
        let baseline = selectsLoadedChange ? selectedChangeIds : selectionBaseline
        guard isRefreshComplete else { return baseline }
        isRefreshingInFlight = false
        fetchPrInfo(bookmarks: selectedChange?.info.bookmarks ?? [])
        resumePendingBackgroundRefresh()
        return baseline
    }

    @MainActor
    private func apply(_ content: RepoRefreshContent, selectsLoadedChange: Bool = true) {
        setGraph(content.graph.entries, graph: content.graph)
        if let context = content.context {
            apply(context)
        }
        if selectsLoadedChange {
            applySingleSelectedChange(content.selectedChange)
        }
        applyWorkingCopy(
            changeId: content.workingCopyChangeId,
            description: content.workingCopyDescription
        )
        isLoading = false
    }

    @MainActor
    func apply(_ context: RepoRefreshContext) {
        bookmarks = context.bookmarks
        if let workspaces = context.workspaces {
            self.workspaces = workspaces
        }
        prHostName = context.prHostName
        apply(context.statusBar)
    }

    func loadMore() {
        guard !isShuttingDown, canLoadMore, let currentDepth = Self.defaultRevsetDepth(for: revset) else { return }

        let nextDepth = currentDepth + Self.defaultRevsetPageSize
        let nextRevset = Self.buildDefaultRevset(depth: nextDepth)
        let previousIds = Set(graphEntries.map(\.change.changeId))
        let preferredRev = selectedChangeId
        let includeSubmoduleStatuses = includeSubmoduleStatuses

        refreshTask?.cancel()
        isRefreshingInFlight = true
        error = nil

        refreshTask = startRepoTask { [weak self, repo, includeSubmoduleStatuses] in
            do {
                let content = try Self.loadRefreshContent(
                    repo: repo,
                    revset: nextRevset,
                    preferredRev: preferredRev,
                    includeSubmoduleStatuses: includeSubmoduleStatuses
                )
                guard !Task.isCancelled else { return }

                let didGrow = !Set(content.graph.entries.map(\.change.changeId)).isSubset(of: previousIds)
                let canLoadMore = didGrow && Self.canLoadMore(
                    revset: nextRevset,
                    loadedCount: content.graph.entries.count
                )

                guard !Task.isCancelled else { return }
                await self?.applyLoadMoreContent(
                    content,
                    canLoadMore: canLoadMore,
                    didGrow: didGrow,
                    revset: nextRevset
                )
            } catch {
                guard !Task.isCancelled else { return }
                let presence = repo.workspacePresence()
                await self?.applyRefreshFailure(error, presence: presence)
            }
        }
    }

    @MainActor
    private func applyLoadMoreContent(
        _ content: RepoRefreshContent,
        canLoadMore: Bool,
        didGrow: Bool,
        revset: String
    ) {
        guard !isShuttingDown else { return }
        apply(content)
        isRefreshingInFlight = false
        self.canLoadMore = canLoadMore
        if didGrow {
            self.revset = revset
        }
        resumePendingBackgroundRefresh()
    }

    func resumePendingBackgroundRefresh(afterFailure: Bool = false) {
        if afterFailure, pendingBackgroundRefresh != nil {
            pendingBackgroundRefresh = .reload
        }
        guard !isShuttingDown, !isBackgroundRefreshSuspended, !isRefreshingInFlight,
              let pending = pendingBackgroundRefresh else { return }
        pendingBackgroundRefresh = nil
        // Compare only after a successful load; a failed load may have advanced the repo without updating the UI.
        if pending == .checkOperation, (try? repo.isAtOperationHead()) == true {
            return
        }
        refresh(isAutoTriggered: true)
    }

    private static func loadRefreshContent(
        repo: JayJayRepo,
        revset: String,
        preferredRev: String?,
        includeSubmoduleStatuses: Bool,
        includeContext: Bool = true
    ) throws -> RepoRefreshContent {
        let graph = try repo.logGraphWithLayout(revset: revset)
        let log = graph.entries.map(\.change)
        let selectedChange = try loadSelectedDetail(
            repo: repo,
            log: log,
            preferredRev: preferredRev,
            includeSubmoduleStatuses: includeSubmoduleStatuses
        )
        let workingCopy = log.first(where: { $0.isWorkingCopy })
        return try RepoRefreshContent(
            graph: graph,
            selectedChange: selectedChange,
            workingCopyChangeId: workingCopy?.changeId.id ?? "",
            workingCopyDescription: workingCopy?.description ?? "",
            context: includeContext ? RepoRefreshContext(repo: repo) : nil
        )
    }
}

extension RepoViewModel {
    /// A clean box follows the working copy; a typed draft is never replaced, even when @ moves to a described change.
    func applyWorkingCopy(changeId: String, description: String) {
        let previousDescription = workingCopyDescription
        workingCopyDescription = description
        guard !changeId.isEmpty else { return }
        let identityChanged = changeId != workingCopyChangeId
        let descriptionChanged = description != previousDescription
        guard identityChanged || descriptionChanged else { return }
        workingCopyChangeId = changeId
        let boxIsClean = commitDraftIsClean(
            summary: commitSummaryDraft,
            body: commitDescriptionDraft,
            message: previousDescription
        )
        guard boxIsClean else { return }
        commitSummaryDraft = commitSummary(message: description)
        commitDescriptionDraft = commitBody(message: description)
    }
}
