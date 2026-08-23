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

    func testFileRollupMapsToChrome() {
        XCTAssertEqual(
            FileRowReviewChrome.chrome(showReview: true, rollup: .reviewed),
            .reviewed
        )
        XCTAssertEqual(
            FileRowReviewChrome.chrome(showReview: true, rollup: .partial),
            .partial
        )
        XCTAssertEqual(
            FileRowReviewChrome.chrome(showReview: true, rollup: .changedSinceReview),
            .changedSinceReview
        )
        XCTAssertEqual(
            FileRowReviewChrome.chrome(showReview: true, rollup: .unreviewed),
            .unreviewed
        )
        XCTAssertEqual(
            FileRowReviewChrome.chrome(showReview: false, rollup: .reviewed),
            .hidden
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
        XCTAssertEqual(row.reviewChrome.systemImage, "circle.fill")
        XCTAssertEqual(row.reviewChrome.accessibilityLabel, "Changed since review")
        XCTAssertFalse(row.showsReviewedStyle)
    }
}
