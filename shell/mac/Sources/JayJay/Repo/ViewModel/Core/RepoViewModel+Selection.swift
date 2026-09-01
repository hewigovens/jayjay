import AppKit
import Foundation
import JayJayCore

extension RepoViewModel {
    /// A held arrow key repeats faster than a detail can load; coalescing loads the change the key settles on.
    static var keyRepeatWindow: Duration {
        .seconds(NSEvent.keyRepeatInterval * 1.5)
    }

    func select(changeId: String?, coalescing: Bool) {
        let wasComparing = compareFromId != nil || compareToId != nil
        comparisonRequestId &+= 1
        compareFromId = nil
        compareToId = nil
        compareDisplay = nil
        let requestedRev = normalizedSelectionRevision(for: changeId)
        selectedChangeId = requestedRev
        selectedChangeIds = requestedRev.map { [$0] } ?? []
        if requestedRev != evologRev {
            evologEntries = nil
            evologRev = nil
        }
        selectionLoadTask?.cancel()
        guard let requestedRev else {
            selectedChange = nil
            return
        }
        if wasComparing {
            selectedChange = nil
        }
        let now = ContinuousClock.now
        let repeating = coalescing && (lastKeyboardSelection.map { now - $0 < Self.keyRepeatWindow } ?? false)
        lastKeyboardSelection = coalescing ? now : nil
        guard repeating else {
            loadSelectedChange(requestedRev)
            return
        }
        selectionLoadTask = Task { [weak self] in
            try? await Task.sleep(for: Self.keyRepeatWindow)
            guard !Task.isCancelled, let self, selectedChangeId == requestedRev else { return }
            loadSelectedChange(requestedRev)
        }
    }

    private func loadSelectedChange(_ requestedRev: String) {
        let includeSubmoduleStatuses = includeSubmoduleStatuses
        runRepoTask {
            try Self.loadSummaryWithConflicts(
                repo: $0,
                rev: requestedRev,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
        } onSuccess: { viewModel, detail in
            guard viewModel.selectedChangeId == requestedRev else { return }
            viewModel.applySingleSelectedChange(detail)
            viewModel.fetchPrInfo(bookmarks: detail.info.bookmarks)
        } onFailure: { viewModel, error in
            guard viewModel.selectedChangeId == requestedRev else { return }
            viewModel.selectedChange = nil
            viewModel.present(error: error)
        }
    }

    func applySingleSelectedChange(_ detail: ChangeDetail?) {
        comparisonRequestId &+= 1
        compareFromId = nil
        compareToId = nil
        compareDisplay = nil
        selectedChange = detail
        selectedChangeId = detail?.info.selectionRevision
        selectedChangeIds = selectedChangeId.map { [$0] } ?? []
    }

    func compareWith(from: String, to: String) {
        compareWith(
            from: from,
            to: to,
            display: RevsetExpressions.compareDisplay(from: from, to: to, changes: changes),
            selectedChangeIds: []
        )
    }

    func toggleSelection(changeId: String) {
        let requestedRev = normalizedSelectionRevision(for: changeId) ?? changeId
        let activeRevisions = selectedChangeIds.isEmpty
            ? selectedChangeId.map { [$0] } ?? []
            : selectedChangeIds
        let orderedRevisions = changes.map(\.selectionRevision)
        var selection = OrderedSelection(
            selectedIDs: Set(activeRevisions),
            primaryID: activeRevisions.first
        )
        selection.apply(.toggle, to: requestedRev, orderedIDs: orderedRevisions)
        let selectedChanges = changes.filter { selection.contains($0.selectionRevision) }
        switch selectedChanges.count {
            case 0:
                select(changeId: nil)
            case 1:
                select(changeId: selectedChanges[0].selectionRevision)
            default:
                guard selection.formsContiguousRange(in: orderedRevisions),
                      DAGViewModel.formsConsecutiveLinearRange(selectedChanges)
                else {
                    showSelectionWithoutDiff(
                        selectedChanges.map(\.selectionRevision),
                        primaryID: selection.primaryID
                    )
                    return
                }
                guard let revsets = combinedDiffRevsets(
                    revisions: selectedChanges.map(\.commitId.id)
                ) else { return }
                compareWith(
                    from: revsets.from,
                    to: revsets.to,
                    display: RevsetExpressions.combinedDiffDisplay(changes: selectedChanges),
                    selectedChangeIds: selectedChanges.map(\.selectionRevision)
                )
        }
    }

    private func showSelectionWithoutDiff(_ selectedIds: [String], primaryID: String?) {
        clearSingleChangePresentation()
        comparisonRequestId &+= 1
        selectionLoadTask?.cancel()
        compareFromId = nil
        compareToId = nil
        compareDisplay = nil
        selectedChange = nil
        selectedChangeId = primaryID ?? selectedIds.first
        selectedChangeIds = selectedIds
    }

    func diffBookmark(_ request: BookmarkDiffRequest) {
        compareWith(
            from: request.compareFromRev,
            to: request.head.rev,
            display: request.display,
            selectedChangeIds: []
        )
    }

    private func compareWith(
        from: String,
        to: String,
        display: CompareDisplay?,
        selectedChangeIds: [String]
    ) {
        clearSingleChangePresentation()
        let fallbackSelectionId = selectedChangeIds.first ?? selectedChangeId
        comparisonRequestId &+= 1
        let requestId = comparisonRequestId
        selectionLoadTask?.cancel()
        compareFromId = from
        compareToId = to
        compareDisplay = display
        self.selectedChangeIds = selectedChangeIds
        selectedChangeId = to
        runRepoTask {
            let detail = try $0.interdiffSummary(fromRev: from, toRev: to)
            // Resolve mutable change IDs so both sides of the cache key are content-addressed and cannot serve a stale interdiff after an amend.
            let fromCommitId = (try? $0.log(revset: from))?.first?.commitId.id
            return (detail, fromCommitId)
        } onSuccess: { viewModel, result in
            guard viewModel.comparisonRequestId == requestId else { return }
            let (detail, fromCommitId) = result
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.selectionRevision
            if let fromCommitId, !fromCommitId.isEmpty {
                viewModel.compareFromId = fromCommitId
            }
        } onFailure: { viewModel, error in
            guard viewModel.comparisonRequestId == requestId else { return }
            viewModel.select(changeId: fallbackSelectionId)
            viewModel.present(error: error)
        }
    }

    private func clearSingleChangePresentation() {
        dismissEvolog()
        clearPrInfo()
    }

    var canReverseCompare: Bool {
        selectedChangeIds.count <= 1 && compareFromId != nil && compareToId != nil
    }

    func reverseCompare() {
        guard canReverseCompare,
              let from = compareFromId,
              let to = compareToId
        else { return }
        let display = compareDisplay.map {
            CompareDisplay(title: $0.title, from: $0.to, to: $0.from)
        }
        compareWith(
            from: to,
            to: from,
            display: display,
            selectedChangeIds: selectedChangeIds
        )
    }

    func clearCompare() {
        select(changeId: selectedChangeId)
    }

    /// Load summary and add working-copy-only shell projections.
    static func loadSummaryWithConflicts(
        repo: JayJayRepo,
        rev: String,
        includeSubmoduleStatuses: Bool = false
    ) throws -> ChangeDetail {
        var detail = try repo.showSummary(rev: rev)
        var hunks = detail.diff
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
                        supportsConflictEditor: hunk.supportsConflictEditor,
                        reviewIdentity: hunk.reviewIdentity,
                        projection: hunk.projection
                    )
                }
            }

            if includeSubmoduleStatuses {
                appendMissingSubmodulePlaceholders(repo: repo, hunks: &hunks)
            }
        }

        detail = ChangeDetail(info: detail.info, diff: hunks)
        return detail
    }

    private static func appendMissingSubmodulePlaceholders(
        repo: JayJayRepo,
        hunks: inout [DiffHunk]
    ) {
        let submoduleStatuses = (try? repo.submoduleStatuses()) ?? []
        let existingPaths = Set(hunks.map(\.path))
        let missing = submoduleStatuses
            .filter { !existingPaths.contains($0.path) }
            .sorted { $0.path < $1.path }
        for status in missing {
            let label = submodulePlaceholderLabel(for: status)
            hunks.append(DiffHunk(
                path: status.path,
                oldPath: nil,
                oldContent: label,
                newContent: label,
                oldPreview: nil,
                newPreview: nil,
                hunkType: .modified,
                reviewIdentity: "",
                projection: nil
            ))
        }
    }

    private static func submodulePlaceholderLabel(for status: GitSubmoduleStatus) -> String {
        if status.hasNewCommits,
           status.hasModifiedContent || status.hasUntrackedContent
        {
            return "<git submodule: updated commit + dirty working tree>"
        }
        if status.hasNewCommits {
            return "<git submodule: updated commit>"
        }
        if status.hasModifiedContent || status.hasUntrackedContent {
            return "<git submodule: dirty working tree>"
        }
        return "<git submodule>"
    }

    static func loadSelectedDetail(
        repo: JayJayRepo,
        log: [ChangeInfo],
        preferredRev: String?,
        includeSubmoduleStatuses: Bool = false
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
            guard let detail = try? loadSummaryWithConflicts(
                repo: repo,
                rev: candidate,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            ) else { continue }
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
