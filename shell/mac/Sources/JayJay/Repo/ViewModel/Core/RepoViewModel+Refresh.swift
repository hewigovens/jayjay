import Foundation
import JayJayCore

private struct RepoRefreshContent {
    let graph: [GraphEntry]
    let bookmarks: [BookmarkInfo]
    let workspaces: [WorkspaceInfo]
    let trunkBookmarkName: String?
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
        snapshotWorkingCopy: Bool = true,
        switchGeneration: UInt64? = nil
    ) {
        // Don't pile FS-triggered refreshes on an in-flight one — our own refreshWorkingCopy re-fires the watcher.
        if isAutoTriggered, isRefreshingInFlight { return }
        refreshTask?.cancel()
        let generation = switchGeneration ?? workspaceSwitchGeneration
        refreshGeneration = generation
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
                            isRefreshComplete: false,
                            generation: generation
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
                        isRefreshComplete: true,
                        generation: generation
                    )
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    guard let self, !self.abandonStaleRefresh(generation: generation) else { return }
                    self.isLoading = false
                    self.isRefreshingInFlight = false
                    self.present(error: error)
                }
            }
        }
    }

    @MainActor
    private func abandonStaleRefresh(generation: UInt64) -> Bool {
        guard workspaceSwitchGeneration != generation else { return false }
        if refreshGeneration == generation {
            isRefreshingInFlight = false
            isLoading = false
        }
        return true
    }

    @MainActor
    private func applyRefreshContent(
        _ content: RepoRefreshContent,
        revset: String,
        isRefreshComplete: Bool,
        generation: UInt64
    ) {
        guard !abandonStaleRefresh(generation: generation) else { return }
        graphEntries = content.graph
        bookmarks = content.bookmarks
        workspaces = WorkspaceSidebarPolicy.mergingAdopted(content.workspaces, current: workspaces)
        trunkBookmarkName = content.trunkBookmarkName
        prHostName = content.prHostName
        let switchPending = workspaces.first(where: \.isCurrent).map {
            !WorkspaceSidebarPolicy.isSamePath($0.path, repoPath)
        } ?? false
        if !switchPending {
            selectedChange = content.selectedChange
            selectedChangeId = content.selectedChange?.info.selectionRevision
            applyWorkingCopy(
                changeId: content.workingCopyChangeId,
                isDivergent: content.workingCopyIsDivergent,
                description: content.workingCopyDescription
            )
        }
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
        let generation = workspaceSwitchGeneration
        refreshGeneration = generation
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
                    guard let self else { return }
                    guard !self.abandonStaleRefresh(generation: generation) else { return }
                    self.graphEntries = content.graph
                    self.bookmarks = content.bookmarks
                    self.workspaces = WorkspaceSidebarPolicy.mergingAdopted(
                        content.workspaces,
                        current: self.workspaces
                    )
                    self.prHostName = content.prHostName
                    self.selectedChange = content.selectedChange
                    self.selectedChangeId = content.selectedChange?.info.selectionRevision
                    self.applyWorkingCopy(
                        changeId: content.workingCopyChangeId,
                        isDivergent: content.workingCopyIsDivergent,
                        description: content.workingCopyDescription
                    )
                    self.apply(content.statusBar)
                    self.isLoading = false
                    self.isRefreshingInFlight = false
                    self.hasWorkingCopyChanges = false
                    self.canLoadMore = canLoadMore
                    if didGrow {
                        self.revset = nextRevset
                    }
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    guard let self, !self.abandonStaleRefresh(generation: generation) else { return }
                    self.isLoading = false
                    self.isRefreshingInFlight = false
                    self.present(error: error)
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
            trunkBookmarkName: repo.trunkBookmarkName(),
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
    /// When @ moves, replace a typed draft only if the new change has a real description.
    func applyWorkingCopy(changeId: String, isDivergent: Bool, description: String) {
        let previousDescription = workingCopyDescription
        workingCopyDescription = description
        guard !changeId.isEmpty else { return }
        let identityChanged = changeId != workingCopyChangeId
        // Divergent siblings share a change id, so a description change is the only signal that @ moved between them.
        guard identityChanged || (isDivergent && description != previousDescription) else { return }
        workingCopyChangeId = changeId
        let hasDraft = !commitSummaryDraft.isEmpty || !commitDescriptionDraft.isEmpty
        if hasDraft, description.isEmpty { return }
        commitSummaryDraft = commitSummary(message: description)
        commitDescriptionDraft = commitBody(message: description)
    }
}
