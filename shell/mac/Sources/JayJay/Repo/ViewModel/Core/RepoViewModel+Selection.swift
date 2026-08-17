import Foundation
import JayJayCore

extension RepoViewModel {
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
        let includeSubmoduleStatuses = includeSubmoduleStatuses

        runRepoTask {
            try Self.loadSummaryWithConflicts(
                repo: $0,
                rev: requestedRev,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
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
        runRepoTask {
            let detail = try $0.interdiffSummary(fromRev: from, toRev: to)
            // Resolve the compare source to its immutable commit id so the diff
            // cache key is content-addressed on both sides; otherwise amending a
            // mutable `from` (a change id) would keep serving a stale interdiff.
            let fromCommitId = (try? $0.log(revset: from))?.first?.commitId.id
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
