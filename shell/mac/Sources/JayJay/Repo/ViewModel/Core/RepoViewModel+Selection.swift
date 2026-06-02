import Foundation
import JayJayCore

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
                    self?.selectedChangeId = detail?.info.selectionRevision
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
                    self?.selectedChangeId = detail?.info.selectionRevision
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
        compareToId = nil
        compareDisplay = nil
        let requestedRev = normalizedSelectionRevision(for: changeId)
        selectedChangeId = requestedRev
        if requestedRev != evologRev {
            evologEntries = nil
            evologRev = nil
        }
        guard let requestedRev else {
            selectedChange = nil
            return
        }
        selectedChange = nil

        load {
            try Self.loadSummaryWithConflicts(repo: $0, rev: requestedRev)
        } onSuccess: { viewModel, detail in
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.selectionRevision
            viewModel.fetchPrInfo(bookmarks: detail.info.bookmarks)
        } onFailure: { viewModel, error in
            if viewModel.selectedChangeId == requestedRev {
                viewModel.selectedChange = nil
            }
            viewModel.present(error: error)
        }
    }

    func compareWith(from: String, to: String) {
        compareWith(
            from: from,
            to: to,
            display: RevsetExpressions.compareDisplay(from: from, to: to, changes: changes)
        )
    }

    func diffBookmark(_ request: BookmarkDiffRequest) {
        compareWith(
            from: request.compareFromRev,
            to: request.head.rev,
            display: request.display
        )
    }

    private func compareWith(
        from: String,
        to: String,
        display: CompareDisplay?
    ) {
        compareFromId = from
        compareToId = to
        compareDisplay = display
        selectedChangeId = to
        load {
            let detail = try $0.interdiffSummary(fromRev: from, toRev: to)
            // Resolve the compare source to its immutable commit id so the diff
            // cache key is content-addressed on both sides; otherwise amending a
            // mutable `from` (a change id) would keep serving a stale interdiff.
            let fromCommitId = (try? $0.log(revset: from))?.first?.commitId
            return (detail, fromCommitId)
        } onSuccess: { viewModel, result in
            let (detail, fromCommitId) = result
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.selectionRevision
            if let fromCommitId, !fromCommitId.isEmpty {
                viewModel.compareFromId = fromCommitId
            }
        } onFailure: { viewModel, error in
            viewModel.compareFromId = nil
            viewModel.compareToId = nil
            viewModel.compareDisplay = nil
            viewModel.present(error: error)
        }
    }

    func reverseCompare() {
        guard let from = compareFromId, let to = compareToId else { return }
        let display = compareDisplay.map {
            CompareDisplay(title: $0.title, from: $0.to, to: $0.from)
        }
        compareWith(
            from: to,
            to: from,
            display: display
        )
    }

    func clearCompare() {
        compareFromId = nil
        compareToId = nil
        compareDisplay = nil
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
                        hunkType: .modified,
                        reviewIdentity: ""
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
                        hunkType: hunk.hunkType,
                        reviewIdentity: hunk.reviewIdentity
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
                    hunkType: .modified,
                    reviewIdentity: ""
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
            let normalized = log.first(where: { $0.matchesRevision(preferredRev) })?.selectionRevision ?? preferredRev
            candidates.append(normalized)
        }
        if let firstRev = log.first?.selectionRevision, !candidates.contains(firstRev) {
            candidates.append(firstRev)
        }

        for candidate in candidates {
            guard let detail = try? loadSummaryWithConflicts(repo: repo, rev: candidate) else { continue }
            if log.contains(where: { $0.commitId == detail.info.commitId }) {
                return detail
            }
        }

        return nil
    }

    private func normalizedSelectionRevision(for rev: String?) -> String? {
        guard let rev else { return nil }
        return changes.first(where: { $0.matchesRevision(rev) })?.selectionRevision ?? rev
    }
}
