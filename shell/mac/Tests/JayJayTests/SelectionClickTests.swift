@testable import JayJay
import JayJayCore
import XCTest

final class SelectionClickTests: XCTestCase {
    func testModifiersDriveTheCoreSelectionAlgebra() {
        let order = ["a", "b", "c"]
        var selection = orderedSelection(selected: ["a"], primary: "a", anchor: "a")

        selection = applySelectionClick(
            selection: selection,
            click: SelectionClick(modifiers: [.shift]),
            id: "c",
            order: order
        )
        XCTAssertEqual(selection.selected, ["a", "b", "c"])

        selection = applySelectionClick(
            selection: selection,
            click: SelectionClick(modifiers: [.command]),
            id: "b",
            order: order
        )
        XCTAssertEqual(orderedSelectionIds(selection: selection, order: order), ["a", "c"])
        XCTAssertFalse(selectionIsContiguous(selection: selection, order: order))

        selection = applySelectionClick(
            selection: selection,
            click: SelectionClick(modifiers: []),
            id: "b",
            order: order
        )
        XCTAssertEqual(selection.selected, ["b"])
    }
}
