import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffSection: DiffGutterReviewActions {
    var reviewModeEnabled: Bool {
        reservesReviewNoteGutterColumn && hunk.projection == nil
    }

    var reservesReviewNoteGutterColumn: Bool {
        Self.reservesReviewNoteGutterColumn(
            isWorkingCopy: isWorkingCopy,
            hasReviewStore: reviewStore != nil,
            reviewChangeId: reviewChangeId,
            reviewIdentity: hunk.reviewIdentity,
            compareFromRev: compareFromRev
        )
    }

    nonisolated static func reservesReviewNoteGutterColumn(
        isWorkingCopy: Bool,
        hasReviewStore: Bool,
        reviewChangeId: String?,
        reviewIdentity: String,
        compareFromRev: String?
    ) -> Bool {
        isWorkingCopy && hasReviewStore && reviewChangeId != nil
            && !reviewIdentity.isEmpty && compareFromRev == nil
    }

    func isHunkReviewed(groupIndex: UInt32) -> Bool {
        guard let reviewStore, let reviewChangeId else { return false }
        return reviewStore.isHunkReviewed(
            changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity, hunkIndex: groupIndex
        )
    }

    func toggleHunkReviewed(groupIndex: UInt32) {
        guard let reviewStore, let reviewChangeId else { return }
        // Demoting a file-marked file: materialize the survivors so we don't drop them all.
        if reviewStore.isReviewed(changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity),
           let total = totalChangeGroupCount, total > 0
        {
            let kept = (0 ..< UInt32(total)).filter { $0 != groupIndex }
            reviewStore.setReviewedHunks(
                changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity, hunkIndices: kept
            )
        } else {
            reviewStore.toggleHunkReviewed(
                changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity, hunkIndex: groupIndex
            )
            promoteFileMarkIfAllReviewed()
        }
        // ChangeDetailView snapshots reviewedPaths in @State via an untracked FFI read, not observation; nudge it to recompute.
        onReviewStateChanged?()
    }

    private func promoteFileMarkIfAllReviewed() {
        guard let reviewStore, let reviewChangeId,
              let total = totalChangeGroupCount, total > 0
        else { return }
        let allReviewed = (0 ..< UInt32(total)).allSatisfy {
            reviewStore.isHunkReviewed(
                changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity, hunkIndex: $0
            )
        }
        let alreadyMarked = reviewStore.isReviewed(
            changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity
        )
        if allReviewed, !alreadyMarked {
            reviewStore.markReviewed(
                changeId: reviewChangeId, path: hunk.path, identity: hunk.reviewIdentity
            )
        }
    }

    /// Contiguous runs of added/removed lines. Nil until the diff has loaded.
    var totalChangeGroupCount: Int? {
        displayGroups?.count
    }
}
