import Foundation
import JayJayCore

private struct RepoRefreshContent {
    let graph: [GraphEntry]
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]
    let prHostName: String?
    let selectedChange: ChangeDetail?
    let workingCopyChangeId: String
    let workingCopyIsDivergent: Bool
    let workingCopyDescription: String
    let statusBar: StatusBarSnapshot
}

extension RepoViewModel {
    func handleWorkingCopyChange() {
        // Ignore the FS echo from our own mutations — perform() already refreshed.
        if let last = lastInternalMutationAt, Date().timeIntervalSince(last) < 5 {
            return
        }
        // If the user is actively reviewing the working copy, don't yank the visible diff.
        if isRepoWindowActive, compareFromId == nil, selectedChange?.info.isWorkingCopy == true {
            hasWorkingCopyChanges = true
            return
        }
        // Elsewhere in the graph: silently update so the WC entry stays current.
        refresh(isAutoTriggered: true)
    }

    func fetchPrInfo(bookmarks: [String]) {
        prFetchTask?.cancel()
        guard let bookmark = bookmarks.first else {
            prInfo = nil
            return
        }
        prInfo = nil
        prFetchTask = Task.detached { [repo] in
            let info = repo.pullRequestInfo(bookmark: bookmark)
            guard !Task.isCancelled else { return }
            await MainActor.run { [weak self] in
                self?.prInfo = info
            }
        }
    }

    func applyRevset(_ newRevset: String) {
        revset = newRevset
        canLoadMore = Self.canLoadMore(revset: newRevset, loadedCount: graphEntries.count)
        refresh(selecting: "@")
    }

    func refresh(
        selecting preferredRev: String? = nil,
        isAutoTriggered: Bool = false,
        snapshotWorkingCopy: Bool = true
    ) {
        // Don't pile FS-triggered refreshes on an in-flight one — our own refreshWorkingCopy re-fires the watcher.
        if isAutoTriggered, isRefreshingInFlight { return }
        refreshTask?.cancel()
        isRefreshingInFlight = true
        isLoading = graphEntries.isEmpty
        hasWorkingCopyChanges = false
        error = nil
        let currentSelection = selectedChangeId
        let requestedRevset = revset
        let includeSubmoduleStatuses = includeSubmoduleStatuses
        let shouldLoadBeforeSnapshot = graphEntries.isEmpty && snapshotWorkingCopy
        refreshTask = Task.detached { [repo] in
            do {
                if shouldLoadBeforeSnapshot {
                    let content = try Self.loadRefreshContent(
                        repo: repo,
                        revset: requestedRevset,
                        preferredRev: preferredRev ?? currentSelection,
                        includeSubmoduleStatuses: includeSubmoduleStatuses
                    )
                    guard !Task.isCancelled else { return }
                    await MainActor.run { [weak self] in
                        self?.applyRefreshContent(
                            content,
                            revset: requestedRevset,
                            isRefreshComplete: false
                        )
                    }
                }

                if snapshotWorkingCopy {
                    try repo.refreshWorkingCopy()
                    guard !Task.isCancelled else { return }
                }

                let content = try Self.loadRefreshContent(
                    repo: repo,
                    revset: requestedRevset,
                    preferredRev: preferredRev ?? currentSelection,
                    includeSubmoduleStatuses: includeSubmoduleStatuses
                )
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.applyRefreshContent(
                        content,
                        revset: requestedRevset,
                        isRefreshComplete: true
                    )
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.isLoading = false
                    self?.isRefreshingInFlight = false
                    self?.present(error: error)
                }
            }
        }
    }

    @MainActor
    private func applyRefreshContent(
        _ content: RepoRefreshContent,
        revset: String,
        isRefreshComplete: Bool
    ) {
        graphEntries = content.graph
        bookmarks = content.bookmarks
        workspaces = content.workspaces
        prHostName = content.prHostName
        selectedChange = content.selectedChange
        selectedChangeId = content.selectedChange?.info.selectionRevision
        applyWorkingCopy(
            changeId: content.workingCopyChangeId,
            isDivergent: content.workingCopyIsDivergent,
            description: content.workingCopyDescription
        )
        apply(content.statusBar)
        isLoading = false
        if isRefreshComplete {
            isRefreshingInFlight = false
            hasWorkingCopyChanges = false
        }
        canLoadMore = Self.canLoadMore(
            revset: revset,
            loadedCount: content.graph.count
        )
        fetchPrInfo(bookmarks: content.selectedChange?.info.bookmarks ?? [])
    }

    func loadMore() {
        guard canLoadMore, let currentDepth = Self.defaultRevsetDepth(for: revset) else { return }

        let nextDepth = currentDepth + Self.defaultRevsetPageSize
        let nextRevset = Self.buildDefaultRevset(depth: nextDepth)
        let previousIds = Set(graphEntries.map(\.change.changeId))
        let preferredRev = selectedChangeId
        let includeSubmoduleStatuses = includeSubmoduleStatuses

        refreshTask?.cancel()
        isRefreshingInFlight = true
        error = nil

        refreshTask = Task.detached { [repo, includeSubmoduleStatuses] in
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
                await MainActor.run { [weak self] in
                    self?.graphEntries = content.graph
                    self?.bookmarks = content.bookmarks
                    self?.workspaces = content.workspaces
                    self?.prHostName = content.prHostName
                    self?.selectedChange = content.selectedChange
                    self?.selectedChangeId = content.selectedChange?.info.selectionRevision
                    self?.applyWorkingCopy(
                        changeId: content.workingCopyChangeId,
                        isDivergent: content.workingCopyIsDivergent,
                        description: content.workingCopyDescription
                    )
                    self?.apply(content.statusBar)
                    self?.isLoading = false
                    self?.isRefreshingInFlight = false
                    self?.hasWorkingCopyChanges = false
                    self?.canLoadMore = canLoadMore
                    if didGrow {
                        self?.revset = nextRevset
                    }
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.isLoading = false
                    self?.isRefreshingInFlight = false
                    self?.present(error: error)
                }
            }
        }
    }

    private static func loadRefreshContent(
        repo: JayJayRepo,
        revset: String,
        preferredRev: String?,
        includeSubmoduleStatuses: Bool
    ) throws -> RepoRefreshContent {
        let graph = try repo.logGraph(revset: revset)
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
            bookmarks: repo.listBookmarks(),
            workspaces: (try? repo.workspaceList()) ?? [],
            prHostName: repo.prHostName(),
            selectedChange: selectedChange,
            workingCopyChangeId: workingCopy?.changeId.id ?? "",
            workingCopyIsDivergent: workingCopy?.isDivergent ?? false,
            workingCopyDescription: workingCopy?.description ?? "",
            statusBar: statusBar
        )
    }
}

extension RepoViewModel {
    /// When @ moves to a different change, reseed the commit-box draft from the new @ description (issue #101); same-change refreshes keep what the user typed, and a typed draft is only replaced when the new @ brings a real description.
    func applyWorkingCopy(changeId: String, isDivergent: Bool, description: String) {
        let previousDescription = workingCopyDescription
        workingCopyDescription = description
        guard !changeId.isEmpty else { return }
        let identityChanged = changeId != workingCopyChangeId
        // Divergent siblings share a change id, so a description change is the only signal that @ moved between them.
        guard identityChanged || (isDivergent && description != previousDescription) else { return }
        workingCopyChangeId = changeId
        let keepDraft = keepCommitDraftOnNextWorkingCopyChange
        keepCommitDraftOnNextWorkingCopyChange = false
        let hasDraft = !commitSummaryDraft.isEmpty || !commitDescriptionDraft.isEmpty
        if hasDraft, keepDraft || description.isEmpty { return }
        commitSummaryDraft = commitSummary(message: description)
        commitDescriptionDraft = commitBody(message: description)
    }
}

/// Status-bar fields a mutation path must refresh alongside the log, so the bar never shows the previous operation or stats until the next full refresh.
struct StatusBarSnapshot {
    let workingCopyStats: DiffStats?
    let currentOperationDescription: String

    static func load(from repo: JayJayRepo) -> StatusBarSnapshot {
        StatusBarSnapshot(
            workingCopyStats: try? repo.diffStats(rev: "@"),
            currentOperationDescription: repo.currentOperationDescription()
        )
    }
}

extension RepoViewModel {
    func apply(_ snapshot: StatusBarSnapshot) {
        workingCopyStats = snapshot.workingCopyStats
        currentOperationDescription = snapshot.currentOperationDescription
    }
}
