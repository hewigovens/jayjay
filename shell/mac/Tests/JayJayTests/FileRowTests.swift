@testable import JayJay
import JayJayCore
import XCTest

final class FileRowTests: XCTestCase {
    private func hunk() -> DiffHunk {
        DiffHunk(
            path: "Sources/App.swift",
            oldPath: nil,
            oldContent: nil,
            newContent: "let value = 1\n",
            oldPreview: nil,
            newPreview: nil,
            hunkType: .added,
            reviewIdentity: "identity"
        )
    }

    func testReviewedStyleOnlyAppliesWhenReviewControlsAreVisible() {
        XCTAssertFalse(
            FileRow(hunk: hunk(), isSelected: false, showReview: false, isReviewed: true).showsReviewedStyle
        )
        XCTAssertFalse(
            FileRow(hunk: hunk(), isSelected: false, showReview: true, isReviewed: false).showsReviewedStyle
        )
        XCTAssertTrue(
            FileRow(hunk: hunk(), isSelected: false, showReview: true, isReviewed: true).showsReviewedStyle
        )
    }
}
