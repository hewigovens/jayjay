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
            FileRow(hunk: hunk(), isSelected: false, showReview: false, reviewRollup: .reviewed).showsReviewedStyle
        )
        XCTAssertFalse(
            FileRow(hunk: hunk(), isSelected: false, showReview: true, reviewRollup: .unreviewed).showsReviewedStyle
        )
        XCTAssertTrue(
            FileRow(hunk: hunk(), isSelected: false, showReview: true, reviewRollup: .reviewed).showsReviewedStyle
        )
    }

    func testRemovedReviewedGroupUsesChangedChrome() {
        let row = FileRow(
            hunk: hunk(),
            isSelected: false,
            showReview: true,
            reviewRollup: .changedSinceReview
        )
        XCTAssertEqual(row.reviewChrome, .changedSinceReview)
        XCTAssertFalse(row.showsReviewedStyle)
    }
}
