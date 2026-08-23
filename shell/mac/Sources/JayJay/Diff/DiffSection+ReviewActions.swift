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
        if let snapshot = loadedDiff?.reviewSnapshot {
            let mapping = loadedDiff?.reviewMapping ?? []
            let states = reviewStore.displayHunkStates(
                changeId: reviewChangeId,
                path: hunk.path,
                identity: hunk.reviewIdentity,
                snapshot: snapshot,
                mapping: mapping
            )
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
        if let snapshot = loadedDiff?.reviewSnapshot {
            reviewStore.toggleDisplayHunk(
                changeId: reviewChangeId,
                path: hunk.path,
                identity: hunk.reviewIdentity,
                snapshot: snapshot,
                mapping: loadedDiff?.reviewMapping ?? [],
                displayIndex: groupIndex
            )
        } else {
            reviewStore.toggleHunkReviewed(
                changeId: reviewChangeId,
                path: hunk.path,
                identity: hunk.reviewIdentity,
                hunkIndex: groupIndex
            )
        }
        onReviewStateChanged?()
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
