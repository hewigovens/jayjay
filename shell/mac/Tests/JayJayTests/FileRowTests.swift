@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class FileRowTests: XCTestCase {
    @MainActor
    func testLongPathsStayOnOneLineAtEachFontSize() throws {
        for size in [13.0, 20.0] {
            for family in try [AppSettings.MonoFont.system, XCTUnwrap(AppSettings.MonoFont(rawValue: "menlo"))] {
                func height(_ path: String, oldPath: String? = nil) throws -> CGFloat {
                    let row = FileRow(
                        hunk: testHunk(path: path, oldPath: oldPath, hunkType: oldPath == nil ? .added : .renamed),
                        isSelected: false
                    )
                    .environment(\.jayjayFontSize, size)
                    .environment(\.jayjayFontFamily, family)
                    .frame(width: 240)
                    return try XCTUnwrap(ImageRenderer(content: row).nsImage).size.height
                }
                XCTAssertEqual(
                    try height("App.swift"),
                    try height("Sources/Deeply/Nested/Directory/RepoContentView+PresentationAndSelection.swift"),
                    accuracy: 1
                )
                XCTAssertEqual(
                    try height("App.swift"),
                    try height("Sources/New/Directory/RenamedFile.swift", oldPath: "Sources/Old/Directory/OriginalFile.swift"),
                    accuracy: 1
                )
            }
        }
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
