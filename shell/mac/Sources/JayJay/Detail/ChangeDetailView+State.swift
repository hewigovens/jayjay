import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func resetState(preservingFileContext: Bool = false) {
        let previousSelectedPath = selectedPath
        let previousSelectedPaths = selectedPaths
        let previousAnchorPath = fileSelectionAnchorPath
        let previousShowFileFilter = showFileFilter
        let previousFileFilter = fileFilter

        descriptionText = detail.info.description
        editingDescription = false
        trackedGitLfsPaths = []
        restoreFileSelection(
            preserving: preservingFileContext,
            previousSelectedPath: previousSelectedPath,
            previousSelectedPaths: previousSelectedPaths,
            previousAnchorPath: previousAnchorPath
        )
        if preservingFileContext {
            showFileFilter = previousShowFileFilter
            fileFilter = previousFileFilter
        } else {
            showFileFilter = false
            fileFilter = ""
        }
        paneMode = .files
        loadConflictedPaths()
        loadTrackedGitLfsPaths()
        loadDiffStats()
        refreshReviewState()
        // No clear(): content-addressed by commit id, so prior changes stay warm and never go stale.
        diffStore.preload(
            hunks: detail.diff,
            rev: detailRevision,
            commitId: detail.info.commitId.id,
            repo: repo,
            compareFromRev: compareFromId,
            ignoreWhitespace: appSettings.ignoreWhitespace
        )
    }

    private func restoreFileSelection(
        preserving: Bool,
        previousSelectedPath: String?,
        previousSelectedPaths: Set<String>,
        previousAnchorPath: String?
    ) {
        let availablePaths = Set(detail.diff.map(\.path))
        let fallbackPath = detail.diff.first?.path

        if preserving,
           let previousSelectedPath,
           availablePaths.contains(previousSelectedPath)
        {
            selectedPath = previousSelectedPath

            let preservedPaths = previousSelectedPaths.intersection(availablePaths)
            selectedPaths = preservedPaths.isEmpty ? [previousSelectedPath] : preservedPaths

            if let previousAnchorPath, availablePaths.contains(previousAnchorPath) {
                fileSelectionAnchorPath = previousAnchorPath
            } else {
                fileSelectionAnchorPath = previousSelectedPath
            }
            return
        }

        selectedPath = fallbackPath
        selectedPaths = fallbackPath.map { [$0] } ?? []
        fileSelectionAnchorPath = fallbackPath
    }

    func loadDiffStats() {
        let rev = detailRevision
        // Key on commitId, not the (stable) changeId, so amends to a mutable change reload.
        let commitId = detail.info.commitId.id
        guard diffStatsCommitId != commitId else { return }
        diffStatsCommitId = commitId
        diffStats = nil
        guard let repo else { return }
        Task.detached {
            let stats = try? repo.diffStats(rev: rev)
            await MainActor.run {
                guard diffStatsCommitId == commitId else { return }
                diffStats = stats
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
                   let selectedPath,
                   trackedGitLfsPaths.contains(selectedPath)
                {
                    let nextVisible = detail.diff.first { !trackedGitLfsPaths.contains($0.path) }
                    self.selectedPath = nextVisible?.path
                    self.selectedPaths = nextVisible.map { [$0.path] } ?? []
                    self.fileSelectionAnchorPath = nextVisible?.path
                }
            }
        }
    }
}
