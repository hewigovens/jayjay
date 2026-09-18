@testable import JayJay
import XCTest

final class DAGRefChipTests: XCTestCase {
    func testChipsKeepStatusAheadOfNamesSoNamesSpillFirst() {
        let change = mockChangeInfo(
            bookmarks: ["main", "feature"],
            tags: ["v1.0"],
            workspaces: ["review"],
            isWorkingCopy: true,
            hasConflict: true,
            isDivergent: true
        )

        XCTAssertEqual(DAGRefChip.chips(for: change), [
            .workingCopy,
            .conflict,
            .divergent,
            .bookmark("main"),
            .bookmark("feature"),
            .gitTag("v1.0"),
            .workspace("review")
        ])
    }

    func testHiddenChipsLabelThemselvesForTheSpillTooltip() {
        let chips = DAGRefChip.chips(for: mockChangeInfo(
            bookmarks: ["main", "markdownlint-increment"],
            workspaces: ["review"],
            isWorkingCopy: true
        ))

        XCTAssertEqual(chips.dropFirst(2).map(\.label), ["markdownlint-increment", "review@"])
    }
}
