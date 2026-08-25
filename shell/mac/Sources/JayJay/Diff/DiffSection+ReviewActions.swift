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

    func hunkReviewState(groupIndex: UInt32) -> DiffGutterHunkReviewState {
        guard let reviewStore, let reviewChangeId else { return .unreviewed }
        if let query = loadedDiff?.reviewQuery {
            let states = reviewStore.displayHunkStates(changeId: reviewChangeId, query: query)
            guard groupIndex < states.count else { return .unreviewed }
            return DiffGutterHunkReviewState(states[Int(groupIndex)])
        }
        return reviewStore.isHunkReviewed(
            changeId: reviewChangeId,
            path: hunk.path,
            identity: hunk.reviewIdentity,
            hunkIndex: groupIndex
        ) ? .reviewed : .unreviewed
    }

    func toggleHunkReviewed(groupIndex: UInt32) {
        guard let reviewStore, let reviewChangeId else { return }
        if let query = loadedDiff?.reviewQuery {
            reviewStore.toggleDisplayHunk(changeId: reviewChangeId, query: query, displayIndex: groupIndex)
        } else {
            toggleLegacyHunk(reviewStore, changeId: reviewChangeId, groupIndex: groupIndex)
        }
        onReviewStateChanged?()
    }

    private func toggleLegacyHunk(
        _ reviewStore: ReviewStore,
        changeId: String,
        groupIndex: UInt32
    ) {
        if reviewStore.isReviewed(
            changeId: changeId,
            path: hunk.path,
            identity: hunk.reviewIdentity
        ), let total = displayGroups?.count, total > 0 {
            reviewStore.setReviewedHunks(
                changeId: changeId,
                path: hunk.path,
                identity: hunk.reviewIdentity,
                hunkIndices: (0 ..< UInt32(total)).filter { $0 != groupIndex }
            )
            return
        }

        reviewStore.toggleHunkReviewed(
            changeId: changeId,
            path: hunk.path,
            identity: hunk.reviewIdentity,
            hunkIndex: groupIndex
        )
        guard let total = displayGroups?.count, total > 0 else { return }
        let allReviewed = (0 ..< UInt32(total)).allSatisfy { index in
            reviewStore.isHunkReviewed(
                changeId: changeId,
                path: hunk.path,
                identity: hunk.reviewIdentity,
                hunkIndex: index
            )
        }
        if allReviewed {
            reviewStore.markReviewed(
                changeId: changeId,
                path: hunk.path,
                identity: hunk.reviewIdentity
            )
        }
    }
}

extension DiffGutterHunkReviewState {
    init(_ state: ReviewGroupState) {
        switch state {
            case .reviewed:
                self = .reviewed
            case .unreviewed:
                self = .unreviewed
            case .changedSinceReview:
                self = .changedSinceReview
            @unknown default:
                self = .unreviewed
        }
    }
}
