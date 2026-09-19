import AppKit
@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class FileRowTests: XCTestCase {
    @MainActor
    func testLongFilenameUsesMoreHeightAndTracksTextSize() {
        func height(path: String, fontSize: Double) -> CGFloat {
            let row = FileRow(hunk: testHunk(path: path), isSelected: true)
                .environment(\.jayjayFontSize, fontSize)
                .frame(width: 240)
                .fixedSize(horizontal: false, vertical: true)
            return NSHostingView(rootView: row).fittingSize.height
        }
        let short = height(path: "Sources/App.swift", fontSize: 13)
        let long = height(path: "Sources/RepoContentView+PresentationAndSelection.swift", fontSize: 13)
        XCTAssertGreaterThan(long, short)
        XCTAssertGreaterThan(height(path: "Sources/RepoContentView+PresentationAndSelection.swift", fontSize: 20), long)
    }

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

    func testAgentBadgeNeedsVisibleControlsAndAMark() {
        XCTAssertFalse(
            FileRow(hunk: hunk(), isSelected: false, showReview: false, reviewRollup: .reviewed, agentMarked: true)
                .showsAgentBadge
        )
        XCTAssertFalse(
            FileRow(hunk: hunk(), isSelected: false, showReview: true, reviewRollup: .unreviewed, agentMarked: true)
                .showsAgentBadge
        )
        let row = FileRow(hunk: hunk(), isSelected: false, showReview: true, reviewRollup: .partial, agentMarked: true)
        XCTAssertTrue(row.showsAgentBadge)
        XCTAssertEqual(row.reviewAccessibilityLabel, "Partially reviewed, includes agent marks")
        XCTAssertEqual(
            FileRow(hunk: hunk(), isSelected: false, showReview: true, reviewRollup: .reviewed).reviewAccessibilityLabel,
            "Reviewed"
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
