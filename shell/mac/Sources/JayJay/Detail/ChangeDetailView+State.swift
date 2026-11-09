import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func resetState(preservingFileContext: Bool = false) {
        trackedGitLfsPaths = []
        fileSelection = preservingFileContext
            ? fileSelection.preserved(in: Set(detail.diff.map(\.path)))
            : FileSelection()
        if !preservingFileContext {
            showFileFilter = false
            fileFilter = ""
            fileEditor = nil
        }
        if !preservingFileContext || conflictEditor?.revision != detailRevision {
            conflictEditor = nil
        }
        paneMode = .files
        let liveSnapshotKeys = Set(detail.diff.map(\.reviewSnapshotKey))
        reviewSnapshots = reviewSnapshots.filter { liveSnapshotKeys.contains($0.key) }
        resolvedReviewSnapshotKeys.formIntersection(liveSnapshotKeys)
        reviewSnapshotRequestId &+= 1
        loadConflictedPaths()
        loadTrackedGitLfsPaths()
        loadDiffStats()
        loadFileStats()
        // No clear(): content-addressed by commit id, so prior changes stay warm and never go stale.
        diffStore.preload(
            hunks: detail.diff,
            rev: detailRevision,
            commitId: detail.info.commitId.id,
            repo: repo,
            compareFromRev: comparison?.fromId,
            ignoreWhitespace: appSettings.ignoreWhitespace
        )
        refreshReviewState()
        reconcileFileSelection()
    }

    func loadDiffStats() {
        let rev = detailRevision
        // Key on commitId, not the (stable) changeId, so amends to a mutable change reload.
        let commitId = detail.info.commitId.id
        guard diffStats?.key != commitId else { return }
        diffStats = KeyedValue(key: commitId)
        guard let repo else { return }
        Task.detached {
            let stats = try? repo.diffStats(rev: rev)
            await MainActor.run {
                guard diffStats?.key == commitId else { return }
                diffStats?.value = stats
            }
        }
    }

    func loadFileStats() {
        // Compare mode shows an interdiff, which the per-revision counts would misstate.
        guard !isCompareMode, let repo else { return }
        let rev = detailRevision
        let ignoreWhitespace = appSettings.ignoreWhitespace
        let key = "\(detail.info.commitId.id)|\(ignoreWhitespace)"
        guard fileStats?.key != key else { return }
        fileStats = KeyedValue(key: key)
        Task.detached {
            let stats = (try? repo.diffFileStats(rev: rev, ignoreWhitespace: ignoreWhitespace)) ?? []
            await MainActor.run {
                guard fileStats?.key == key else { return }
                fileStats?.value = Dictionary(uniqueKeysWithValues: stats.map { ($0.path, $0) })
            }
        }
    }

    func loadTrackedGitLfsPaths() {
        guard let repo, detail.info.isWorkingCopy else {
            trackedGitLfsPaths = []
            return
        }
        let paths = detail.diff.map(\.path)
        Task.detached {
            let paths = (try? repo.gitLfsPaths(paths: paths)) ?? []
            await MainActor.run {
                trackedGitLfsPaths = Set(paths)
                if appSettings.hideGitLfsDiffs,
                   let selectedPath = fileSelection.primary,
                   trackedGitLfsPaths.contains(selectedPath)
                {
                    reconcileFileSelection()
                }
            }
        }
    }
}
