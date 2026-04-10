import JayJayCore

extension RepoViewModel {
    func handleWorkingCopyChange() {
        hasWorkingCopyChanges = true

        guard compareFromId == nil,
              selectedChange?.info.isWorkingCopy == true
        else { return }

        load {
            try $0.refreshWorkingCopy()
            let detail = try Self.loadSummaryWithConflicts(repo: $0, rev: "@")
            return detail
        } onSuccess: { viewModel, detail in
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.changeId
            viewModel.workingCopyDescription = detail.info.description
            viewModel.graphEntries = viewModel.graphEntries.map { entry in
                guard entry.change.isWorkingCopy || entry.change.changeId == detail.info.changeId else {
                    return entry
                }
                return GraphEntry(change: detail.info, edges: entry.edges)
            }
            viewModel.hasWorkingCopyChanges = false
        }
    }

    func applyRevset(_ newRevset: String) {
        revset = newRevset
        canLoadMore = Self.canLoadMore(revset: newRevset, loadedCount: graphEntries.count)
        refresh(selecting: "@")
    }

    func refresh(selecting preferredRev: String? = nil) {
        refreshTask?.cancel()
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
                    self?.hasWorkingCopyChanges = false
                    self?.canLoadMore = Self.canLoadMore(
                        revset: requestedRevset,
                        loadedCount: graph.count
                    )
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.isLoading = false
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
                let label: String
                if status.hasNewCommits,
                   status.hasModifiedContent || status.hasUntrackedContent
                {
                    label = "<git submodule: updated commit + dirty working tree>"
                } else if status.hasNewCommits {
                    label = "<git submodule: updated commit>"
                } else if status.hasModifiedContent || status.hasUntrackedContent {
                    label = "<git submodule: dirty working tree>"
                } else {
                    label = "<git submodule>"
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
