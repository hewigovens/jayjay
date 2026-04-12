import Foundation
import JayJayCore

extension RepoViewModel {
    func handleWorkingCopyChange() {
        // Ignore the FS echo from our own mutations — perform() already refreshed.
        if let last = lastInternalMutationAt, Date().timeIntervalSince(last) < 5 {
            return
        }
        // Reviewing the working copy: flag the refresh button instead of yanking the review.
        if compareFromId == nil, selectedChange?.info.isWorkingCopy == true {
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
            let info = repo.ghPrInfo(bookmark: bookmark)
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

    func refresh(selecting preferredRev: String? = nil, isAutoTriggered: Bool = false) {
        // Don't pile FS-triggered refreshes on an in-flight one — our own refreshWorkingCopy re-fires the watcher.
        if isAutoTriggered, isRefreshingInFlight { return }
        refreshTask?.cancel()
        isRefreshingInFlight = true
        isLoading = graphEntries.isEmpty
        hasWorkingCopyChanges = false
        error = nil
        let currentSelection = selectedChangeId
        let requestedRevset = revset
        refreshTask = Task.detached { [repo, requestedRevset] in
            do {
                try repo.refreshWorkingCopy()
                guard !Task.isCancelled else { return }

                let graph: [GraphEntry]
                do {
                    graph = try repo.logGraph(revset: requestedRevset)
                } catch {
                    guard !Task.isCancelled else { return }
                    await MainActor.run { [weak self] in
                        self?.graphEntries = []
                        self?.selectedChange = nil
                        self?.selectedChangeId = nil
                        self?.isLoading = false
                        self?.isRefreshingInFlight = false
                        self?.present(error: error)
                    }
                    return
                }

                guard !Task.isCancelled else { return }

                let log = graph.map(\.change)
                let marks = try repo.listBookmarks()
                let wsList = (try? repo.workspaceList()) ?? []
                let detail = try Self.loadSelectedDetail(
                    repo: repo,
                    log: log,
                    preferredRev: preferredRev ?? currentSelection
                )
                let wcDesc = log.first(where: { $0.isWorkingCopy })?.description ?? ""
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.graphEntries = graph
                    self?.bookmarks = marks
                    self?.workspaces = wsList
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail?.info.changeId
                    self?.workingCopyDescription = wcDesc
                    self?.isLoading = false
                    self?.isRefreshingInFlight = false
                    self?.hasWorkingCopyChanges = false
                    self?.canLoadMore = Self.canLoadMore(
                        revset: requestedRevset,
                        loadedCount: graph.count
                    )
                    self?.fetchPrInfo(bookmarks: detail?.info.bookmarks ?? [])
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

    func loadMore() {
        guard canLoadMore, let currentDepth = Self.defaultRevsetDepth(for: revset) else { return }

        let nextDepth = currentDepth + Self.defaultRevsetPageSize
        let nextRevset = Self.buildDefaultRevset(depth: nextDepth)
        let previousIds = Set(graphEntries.map(\.change.changeId))
        let preferredRev = selectedChangeId

        refreshTask?.cancel()
        isRefreshingInFlight = true
        error = nil

        refreshTask = Task.detached { [repo] in
            do {
                try repo.refreshWorkingCopy()
                guard !Task.isCancelled else { return }

                let graph = try repo.logGraph(revset: nextRevset)
                guard !Task.isCancelled else { return }

                let log = graph.map(\.change)
                let marks = try repo.listBookmarks()
                let wsList = (try? repo.workspaceList()) ?? []
                let detail = try Self.loadSelectedDetail(
                    repo: repo,
                    log: log,
                    preferredRev: preferredRev
                )
                let wcDesc = log.first(where: { $0.isWorkingCopy })?.description ?? ""
                let didGrow = !Set(log.map(\.changeId)).isSubset(of: previousIds)
                let canLoadMore = didGrow && Self.canLoadMore(
                    revset: nextRevset,
                    loadedCount: graph.count
                )

                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.graphEntries = graph
                    self?.bookmarks = marks
                    self?.workspaces = wsList
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail?.info.changeId
                    self?.workingCopyDescription = wcDesc
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

    func select(changeId: String?) {
        compareFromId = nil
        selectedChangeId = changeId
        guard let changeId else {
            selectedChange = nil
            return
        }
        selectedChange = nil

        load {
            try Self.loadSummaryWithConflicts(repo: $0, rev: changeId)
        } onSuccess: { viewModel, detail in
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.changeId
            viewModel.fetchPrInfo(bookmarks: detail.info.bookmarks)
        } onFailure: { viewModel, error in
            if viewModel.selectedChangeId == changeId {
                viewModel.selectedChange = nil
            }
            viewModel.present(error: error)
        }
    }

    func compareWith(from: String, to: String) {
        compareFromId = from
        selectedChangeId = to
        load {
            try $0.interdiffSummary(fromRev: from, toRev: to)
        } onSuccess: { viewModel, detail in
            viewModel.selectedChange = detail
        } onFailure: { viewModel, error in
            viewModel.compareFromId = nil
            viewModel.present(error: error)
        }
    }

    func clearCompare() {
        compareFromId = nil
        if let selectedChangeId {
            select(changeId: selectedChangeId)
        }
    }

    /// Load summary and merge any conflicted files that don't appear in the normal diff.
    static func loadSummaryWithConflicts(repo: JayJayRepo, rev: String) throws -> ChangeDetail {
        var detail = try repo.showSummary(rev: rev)
        var hunks = detail.diff
        if detail.info.hasConflict {
            let conflictPaths = (try? repo.resolveList(rev: rev)) ?? []
            let existingPaths = Set(hunks.map(\.path))
            let missing = conflictPaths.filter { !existingPaths.contains($0) }
            if !missing.isEmpty {
                for path in missing {
                    hunks.append(DiffHunk(
                        path: path,
                        oldPath: nil,
                        oldContent: nil,
                        newContent: nil,
                        oldPreview: nil,
                        newPreview: nil,
                        hunkType: .modified
                    ))
                }
            }
        }

        if detail.info.isWorkingCopy {
            let trackedGitLfsPaths = Set((try? repo.gitLfsPaths(paths: hunks.map(\.path))) ?? [])
            if !trackedGitLfsPaths.isEmpty {
                hunks = hunks.map { hunk in
                    guard trackedGitLfsPaths.contains(hunk.path) else { return hunk }
                    return DiffHunk(
                        path: hunk.path,
                        oldPath: hunk.oldPath,
                        oldContent: "<git lfs tracked file>",
                        newContent: "<git lfs tracked file>",
                        oldPreview: nil,
                        newPreview: nil,
                        hunkType: hunk.hunkType
                    )
                }
            }

            let submoduleStatuses = (try? repo.submoduleStatuses()) ?? []
            let existingPaths = Set(hunks.map(\.path))
            let missing = submoduleStatuses
                .filter { !existingPaths.contains($0.path) }
                .sorted { $0.path < $1.path }
            for status in missing {
                let label = if status.hasNewCommits,
                               status.hasModifiedContent || status.hasUntrackedContent
                {
                    "<git submodule: updated commit + dirty working tree>"
                } else if status.hasNewCommits {
                    "<git submodule: updated commit>"
                } else if status.hasModifiedContent || status.hasUntrackedContent {
                    "<git submodule: dirty working tree>"
                } else {
                    "<git submodule>"
                }

                hunks.append(DiffHunk(
                    path: status.path,
                    oldPath: nil,
                    oldContent: label,
                    newContent: label,
                    oldPreview: nil,
                    newPreview: nil,
                    hunkType: .modified
                ))
            }
        }

        detail = ChangeDetail(info: detail.info, diff: hunks)
        return detail
    }

    static func loadSelectedDetail(
        repo: JayJayRepo,
        log: [ChangeInfo],
        preferredRev: String?
    ) throws -> ChangeDetail? {
        var candidates = [String]()
        if let preferredRev, !preferredRev.isEmpty {
            candidates.append(preferredRev)
        }
        if let firstChange = log.first?.changeId, !candidates.contains(firstChange) {
            candidates.append(firstChange)
        }

        for candidate in candidates {
            guard let detail = try? loadSummaryWithConflicts(repo: repo, rev: candidate) else { continue }
            if log.contains(where: { $0.changeId == detail.info.changeId }) {
                return detail
            }
        }

        return nil
    }
}
