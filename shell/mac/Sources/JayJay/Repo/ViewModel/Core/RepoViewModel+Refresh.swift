import Foundation
import JayJayCore

private struct RepoRefreshContent {
    let graph: [GraphEntry]
    let dagLayout: DAGLayout
    let hasMore: Bool
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]?
    let prHostName: String?
    let selectedChange: ChangeDetail?
    let workingCopyChangeId: String
    let workingCopyDescription: String
    let statusBar: StatusBarSnapshot
}

extension RepoViewModel {
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

    func applyRevset(_ newRevset: String) {
        let trimmed = newRevset.trimmingCharacters(in: .whitespacesAndNewlines)
        isDefaultRevset = trimmed.isEmpty
        revset = trimmed.isEmpty ? Self.buildDefaultRevset() : trimmed
        appliedLimit = Self.defaultLogPageSize
        canLoadMore = false
        refresh(selecting: "@")
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
        let requestedRevset = currentLogQueryRevset
        let requestedLimit = appliedLimit
        let includeSubmoduleStatuses = includeSubmoduleStatuses
        let shouldLoadBeforeSnapshot = graphEntries.isEmpty && snapshotWorkingCopy
        refreshTask = startRepoTask { [weak self, repo] in
            do {
                if shouldLoadBeforeSnapshot {
                    let content = try Self.loadRefreshContent(
                        repo: repo,
                        revset: requestedRevset,
                        limit: requestedLimit,
                        preferredRev: preferredRev ?? currentSelection,
                        includeSubmoduleStatuses: includeSubmoduleStatuses
                    )
                    guard !Task.isCancelled else { return }
                    await self?.applyRefreshContent(
                        content,
                        isRefreshComplete: false,
                        isAutoTriggered: isAutoTriggered
                    )
                }

                if snapshotWorkingCopy {
                    try repo.refreshWorkingCopy()
                    guard !Task.isCancelled else { return }
                }

                let content = try Self.loadRefreshContent(
                    repo: repo,
                    revset: requestedRevset,
                    limit: requestedLimit,
                    preferredRev: preferredRev ?? currentSelection,
                    includeSubmoduleStatuses: includeSubmoduleStatuses
                )
                guard !Task.isCancelled else { return }
                await self?.applyRefreshContent(
                    content,
                    isRefreshComplete: true,
                    isAutoTriggered: isAutoTriggered
                )
            } catch {
                guard !Task.isCancelled else { return }
                let presence = repo.workspacePresence()
                await self?.applyRefreshFailure(error, presence: presence)
            }
        }
    }

    @MainActor
    private func applyRefreshContent(
        _ content: RepoRefreshContent,
        isRefreshComplete: Bool,
        isAutoTriggered: Bool
    ) {
        guard !isShuttingDown else { return }
        if isAutoTriggered, isBackgroundRefreshSuspended {
            hasPendingBackgroundRefresh = true
            if isRefreshComplete {
                isRefreshingInFlight = false
            }
            return
        }
        graphEntries = content.graph
        dagLayout = content.dagLayout
        bookmarks = content.bookmarks
        if let workspaces = content.workspaces {
            self.workspaces = workspaces
        }
        prHostName = content.prHostName
        applySingleSelectedChange(content.selectedChange)
        applyWorkingCopy(
            changeId: content.workingCopyChangeId,
            description: content.workingCopyDescription
        )
        apply(content.statusBar)
        isLoading = false
        if isRefreshComplete {
            isRefreshingInFlight = false
        }
        canLoadMore = content.hasMore
        fetchPrInfo(bookmarks: content.selectedChange?.info.bookmarks ?? [])
        if isRefreshComplete {
            resumePendingBackgroundRefresh()
        }
    }

    func loadMore() {
        guard !isShuttingDown, canLoadMore else { return }

        let nextLimit = appliedLimit + Self.defaultLogPageSize
        let currentRevset = currentLogQueryRevset
        let preferredRev = selectedChangeId
        let includeSubmoduleStatuses = includeSubmoduleStatuses

        refreshTask?.cancel()
        isRefreshingInFlight = true
        error = nil

        refreshTask = startRepoTask { [weak self, repo, includeSubmoduleStatuses] in
            do {
                let content = try Self.loadRefreshContent(
                    repo: repo,
                    revset: currentRevset,
                    limit: nextLimit,
                    preferredRev: preferredRev,
                    includeSubmoduleStatuses: includeSubmoduleStatuses
                )
                guard !Task.isCancelled else { return }
                await self?.applyLoadMoreContent(content, appliedLimit: nextLimit)
            } catch {
                guard !Task.isCancelled else { return }
                let presence = repo.workspacePresence()
                await self?.applyRefreshFailure(error, presence: presence)
            }
        }
    }

    @MainActor
    private func applyLoadMoreContent(_ content: RepoRefreshContent, appliedLimit: Int) {
        guard !isShuttingDown else { return }
        graphEntries = content.graph
        dagLayout = content.dagLayout
        bookmarks = content.bookmarks
        if let workspaces = content.workspaces {
            self.workspaces = workspaces
        }
        prHostName = content.prHostName
        applySingleSelectedChange(content.selectedChange)
        applyWorkingCopy(
            changeId: content.workingCopyChangeId,
            description: content.workingCopyDescription
        )
        apply(content.statusBar)
        isLoading = false
        isRefreshingInFlight = false
        self.appliedLimit = appliedLimit
        canLoadMore = content.hasMore
        resumePendingBackgroundRefresh()
    }

    func resumePendingBackgroundRefresh() {
        guard !isBackgroundRefreshSuspended, hasPendingBackgroundRefresh else { return }
        hasPendingBackgroundRefresh = false
        refresh(isAutoTriggered: true)
    }

    private static func loadRefreshContent(
        repo: JayJayRepo,
        revset: String?,
        limit: Int,
        preferredRev: String?,
        includeSubmoduleStatuses: Bool
    ) throws -> RepoRefreshContent {
        let page = try repo.logGraphPage(revset: revset, limit: UInt32(limit))
        let graph = page.entries
        let dagLayout = DAGLayout(computed: page.layout)
        let log = graph.map(\.change)
        let selectedChange = try loadSelectedDetail(
            repo: repo,
            log: log,
            preferredRev: preferredRev,
            includeSubmoduleStatuses: includeSubmoduleStatuses
        )
        let statusBar = StatusBarSnapshot.load(from: repo)
        let workingCopy = log.first(where: { $0.isWorkingCopy })
        return try RepoRefreshContent(
            graph: graph,
            dagLayout: dagLayout,
            hasMore: page.hasMore,
            bookmarks: repo.listBookmarks(),
            workspaces: try? repo.workspaceList(),
            prHostName: repo.prHostName(),
            selectedChange: selectedChange,
            workingCopyChangeId: workingCopy?.changeId.id ?? "",
            workingCopyDescription: workingCopy?.description ?? "",
            statusBar: statusBar
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
