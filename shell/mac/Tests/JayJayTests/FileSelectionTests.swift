@testable import JayJay
import XCTest

final class FileSelectionTests: XCTestCase {
    func testRevealingAFileOpensItsAncestorsAndReplacesTheSelection() {
        var collapsed: Set = ["src", "src/lib", "src/library", "docs"]
        var selection = FileSelection.single("other")
        selection.reveal("src/lib/file.swift", expanding: &collapsed)
        XCTAssertEqual(collapsed, ["src/library", "docs"])
        XCTAssertEqual(selection, .single("src/lib/file.swift"))
    }

    func testPreservedSelectionKeepsSurvivingFilesOnlyWhilePrimarySurvives() {
        let selection = FileSelection(primary: "a", paths: ["a", "b", "c"], anchor: "c")

        XCTAssertEqual(selection.preserved(in: ["a", "b"]), FileSelection(primary: "a", paths: ["a", "b"], anchor: "a"))
        XCTAssertEqual(selection.preserved(in: ["b", "c"]), FileSelection())
    }
}
