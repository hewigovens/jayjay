import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffSection: DiffGutterReviewActions {
    var reviewCheckboxesEnabled: Bool {
        // Working-copy only; off in interdiff/compare mode.
        isWorkingCopy && reviewStore != nil && rev != nil
            && !hunk.reviewIdentity.isEmpty && compareFromRev == nil
    }

    func isHunkReviewed(groupIndex: UInt32) -> Bool {
        guard let reviewStore, let rev else { return false }
        return reviewStore.isHunkReviewed(
            changeId: rev, path: hunk.path, identity: hunk.reviewIdentity, hunkIndex: groupIndex
        )
    }

    func toggleHunkReviewed(groupIndex: UInt32) {
        guard let reviewStore, let rev else { return }
        // Demoting a file-marked file: materialize the survivors so we don't drop them all.
        if reviewStore.isReviewed(changeId: rev, path: hunk.path, identity: hunk.reviewIdentity),
           let total = totalChangeGroupCount, total > 0
        {
            let kept = (0 ..< UInt32(total)).filter { $0 != groupIndex }
            reviewStore.setReviewedHunks(
                changeId: rev, path: hunk.path, identity: hunk.reviewIdentity, hunkIndices: kept
            )
        } else {
            reviewStore.toggleHunkReviewed(
                changeId: rev, path: hunk.path, identity: hunk.reviewIdentity, hunkIndex: groupIndex
            )
            promoteFileMarkIfAllReviewed()
        }
        // ChangeDetailView caches reviewedPaths in @State; nudge it to recompute.
        onReviewStateChanged?()
    }

    private func promoteFileMarkIfAllReviewed() {
        guard let reviewStore, let rev,
              let total = totalChangeGroupCount, total > 0
        else { return }
        let allReviewed = (0 ..< UInt32(total)).allSatisfy {
            reviewStore.isHunkReviewed(
                changeId: rev, path: hunk.path, identity: hunk.reviewIdentity, hunkIndex: $0
            )
        }
        let alreadyMarked = reviewStore.isReviewed(
            changeId: rev, path: hunk.path, identity: hunk.reviewIdentity
        )
        if allReviewed, !alreadyMarked {
            reviewStore.markReviewed(
                changeId: rev, path: hunk.path, identity: hunk.reviewIdentity
            )
        }
    }

    /// Contiguous runs of added/removed lines. Nil until the diff has loaded.
    var totalChangeGroupCount: Int? {
        guard let lines = fileDiff?.lines else { return nil }
        var count = 0
        var inGroup = false
        for line in lines {
            let isChanged = line.style == .added || line.style == .removed
            if isChanged, !inGroup {
                count += 1
                inGroup = true
            } else if !isChanged, inGroup {
                inGroup = false
            }
        }
        return count
    }
}
