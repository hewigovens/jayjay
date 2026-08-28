@testable import JayJay
import XCTest

final class OrderedSelectionTests: XCTestCase {
    func testReplaceToggleAndRangePreserveVisibleOrder() {
        let order = ["a", "b", "c", "d"]
        var selection = OrderedSelection<String>()

        selection.apply(.replace, to: "b", orderedIDs: order)
        selection.apply(.toggle, to: "b", orderedIDs: order)
        XCTAssertTrue(selection.selectedIDs.isEmpty)
        XCTAssertNil(selection.primaryID)

        selection.apply(.replace, to: "b", orderedIDs: order)
        selection.apply(.toggle, to: "d", orderedIDs: order)
        XCTAssertEqual(selection.orderedIDs(in: order), ["b", "d"])

        selection.apply(.toggle, to: "d", orderedIDs: order)
        XCTAssertEqual(selection.primaryID, "b")

        selection.apply(.toggle, to: "d", orderedIDs: order)
        selection.apply(.extend, to: "b", orderedIDs: order)
        XCTAssertEqual(selection.orderedIDs(in: order), ["b", "c", "d"])
        XCTAssertEqual(selection.anchorID, "d")
        XCTAssertEqual(selection.primaryID, "b")
    }

    func testRecognizesOnlySelectionsWithoutGapsAsContiguous() {
        let order = ["a", "b", "c", "d"]
        var selection = OrderedSelection(selectedIDs: ["a"], primaryID: "a")

        selection.apply(.toggle, to: "c", orderedIDs: order)
        XCTAssertFalse(selection.formsContiguousRange(in: order))

        selection.apply(.toggle, to: "b", orderedIDs: order)
        XCTAssertTrue(selection.formsContiguousRange(in: order))
    }
}
