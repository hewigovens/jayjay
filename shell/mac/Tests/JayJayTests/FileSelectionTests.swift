@testable import JayJay
import XCTest

final class FileSelectionTests: XCTestCase {
    func testPreservedSelectionKeepsSurvivingFilesOnlyWhilePrimarySurvives() {
        let selection = FileSelection(primary: "a", paths: ["a", "b", "c"], anchor: "c")

        XCTAssertEqual(selection.preserved(in: ["a", "b"]), FileSelection(primary: "a", paths: ["a", "b"], anchor: "a"))
        XCTAssertEqual(selection.preserved(in: ["b", "c"]), FileSelection())
    }
}
