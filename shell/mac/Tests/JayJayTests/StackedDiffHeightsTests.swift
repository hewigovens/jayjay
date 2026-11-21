@testable import JayJay
import XCTest

final class StackedDiffHeightsTests: XCTestCase {
    private func basis(commitId: String = "a", columns: StackedDiffHeights.Columns) -> StackedDiffHeights.Basis {
        .init(commitId: commitId, ignoreWhitespace: false, typography: Typography(), columns: columns)
    }

    func testALayoutChangeKeepsOnlyTheHeightsCarriedThroughIt() {
        let heights = StackedDiffHeights()
        heights.record(100, of: "on-screen", in: basis(columns: .unified))
        heights.record(200, of: "out-of-view", in: basis(columns: .unified))

        heights.carry("on-screen", to: basis(columns: .sideBySide))

        XCTAssertEqual(heights.height(of: "on-screen", in: basis(columns: .sideBySide)), 100)
        XCTAssertNil(heights.height(of: "out-of-view", in: basis(columns: .sideBySide)))
        XCTAssertNil(heights.height(of: "on-screen", in: basis(commitId: "b", columns: .sideBySide)))
    }

    func testADiffThatOnlyRendersUnifiedKeepsItsHeightAcrossTheLayoutToggle() {
        let heights = StackedDiffHeights()
        heights.record(100, of: "added", in: basis(columns: .alwaysUnified))

        XCTAssertEqual(heights.height(of: "added", in: basis(columns: .sideBySide)), 100)
        XCTAssertEqual(heights.height(of: "added", in: basis(columns: .unified)), 100)
    }
}
