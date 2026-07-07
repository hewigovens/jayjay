@testable import JayJay
import JayJayCore
import XCTest

final class FileRowTests: XCTestCase {
    private func hunk() -> DiffHunk {
        testHunk(
            path: "Sources/App.swift",
            newContent: "let value = 1\n",
            hunkType: .added
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
