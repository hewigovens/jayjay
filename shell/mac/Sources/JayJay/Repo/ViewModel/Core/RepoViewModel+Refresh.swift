import Foundation
import JayJayCore

private struct RepoRefreshContent {
    let graph: [GraphEntry]
    let selectedChange: ChangeDetail?
    let workingCopyChangeId: String
    let workingCopyDescription: String
    /// nil on the pre-snapshot pass, which loads only what the first paint needs.
    let context: RepoRefreshContext?
}

/// Repo-wide state around the graph; a cold open paints without it and picks it up on the pass after the snapshot.
struct RepoRefreshContext {
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]?
    let prHostName: String?
    let statusBar: StatusBarSnapshot
}

extension RepoViewModel {
    /// An operation landed. Our own snapshot writes one too and the watcher reports it straight back; when the loaded repo already sits at that head there is nothing to load.
    func handleOperationChange() {
        // Mid-refresh the on-disk head can be ahead of the repo the snapshot is about to install; classify once that refresh has finished.
        if isRefreshingInFlight {
            hasPendingOperationCheck = true
            return
        }
        if (try? repo.isAtOperationHead()) == true {
            return
        }
        handleWorkingCopyChange()
    }

    func handleWorkingCopyChange() {
        guard !isShuttingDown else { return }
        // Remember an event while editing even if a mutation also stamped the echo window; resume without re-checking the stamp once editing ends.
        if isBackgroundRefreshSuspended {
            hasPendingBackgroundRefresh = true
            return
        }
        // Ignore the FS echo from our own mutations — perform() already refreshed.
        if let last = lastInternalMutationAt, Date().timeIntervalSince(last) < 5 {
            return
        }
        refresh(isAutoTriggered: true)
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
        // Don't pile FS-triggered refreshes on an in-flight one — our own refreshWorkingCopy re-fires the watcher.
        if isAutoTriggered, isRefreshingInFlight {
            hasPendingBackgroundRefresh = true
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
        var keepsSelection = preferredRev == nil ? selectedChangeIds : nil
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
                    keepsSelection = await self?.applyRefreshContent(
                        content,
                        revset: requestedRevset,
                        isRefreshComplete: false,
                        isAutoTriggered: isAutoTriggered,
                        keepsSelection: keepsSelection
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
                    isRefreshComplete: true,
                    isAutoTriggered: isAutoTriggered,
                    keepsSelection: keepsSelection
                )
            } catch {
                guard !Task.isCancelled else { return }
                // The first paint went out without repository context; do not leave the window on empty bookmarks and status.
                let context = shouldLoadBeforeSnapshot ? try? Self.loadRefreshContext(repo: repo) : nil
                await self?.applyRefreshFailure(error, presence: repo.workspacePresence(), context: context)
            }
        }
    }

    /// `keepsSelection` is the selection this refresh last installed, or nil until a requested revision has been applied; a selection the user changed since then, such as a multi-selection made after the first paint, stays as it is. Returns the baseline for the next pass.
    @MainActor
    @discardableResult
    private func applyRefreshContent(
        _ content: RepoRefreshContent,
        revset: String,
        isRefreshComplete: Bool,
        isAutoTriggered: Bool,
        keepsSelection: [String]?
    ) -> [String]? {
        guard !isShuttingDown else { return keepsSelection }
        if isAutoTriggered, isBackgroundRefreshSuspended {
            hasPendingBackgroundRefresh = true
            if isRefreshComplete {
                isRefreshingInFlight = false
            }
            return keepsSelection
        }
        let selectsLoadedChange = keepsSelection == nil || keepsSelection == selectedChangeIds
        apply(content, selectsLoadedChange: selectsLoadedChange)
        canLoadMore = Self.canLoadMore(
            revset: revset,
            loadedCount: content.graph.count
        )
        let baseline = selectsLoadedChange ? selectedChangeIds : keepsSelection
        guard isRefreshComplete else { return baseline }
        isRefreshingInFlight = false
        fetchPrInfo(bookmarks: selectedChange?.info.bookmarks ?? [])
        resumePendingBackgroundRefresh()
        return baseline
    }

    @MainActor
    private func apply(_ content: RepoRefreshContent, selectsLoadedChange: Bool = true) {
        graphEntries = content.graph
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

                let didGrow = !Set(content.graph.map(\.change.changeId)).isSubset(of: previousIds)
                let canLoadMore = didGrow && Self.canLoadMore(
                    revset: nextRevset,
                    loadedCount: content.graph.count
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

    func resumePendingBackgroundRefresh() {
        if hasPendingOperationCheck {
            hasPendingOperationCheck = false
            handleOperationChange()
        }
        guard !isBackgroundRefreshSuspended, hasPendingBackgroundRefresh else { return }
        hasPendingBackgroundRefresh = false
        refresh(isAutoTriggered: true)
    }

    private static func loadRefreshContent(
        repo: JayJayRepo,
        revset: String,
        preferredRev: String?,
        includeSubmoduleStatuses: Bool,
        includeContext: Bool = true
    ) throws -> RepoRefreshContent {
        let graph = try repo.logGraph(revset: revset)
        let log = graph.map(\.change)
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
            context: includeContext ? loadRefreshContext(repo: repo) : nil
        )
    }

    private static func loadRefreshContext(repo: JayJayRepo) throws -> RepoRefreshContext {
        try RepoRefreshContext(
            bookmarks: repo.listBookmarks(),
            workspaces: try? repo.workspaceList(),
            prHostName: repo.prHostName(),
            statusBar: StatusBarSnapshot.load(from: repo)
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
        let boxIsClean = commitSummaryDraft == commitSummary(message: previousDescription)
            && commitDescriptionDraft == commitBody(message: previousDescription)
        guard boxIsClean else { return }
        commitSummaryDraft = commitSummary(message: description)
        commitDescriptionDraft = commitBody(message: description)
    }
}
