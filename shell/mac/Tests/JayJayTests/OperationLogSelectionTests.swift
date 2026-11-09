@testable import JayJay
import JayJayCore
import XCTest

final class OperationLogSelectionTests: XCTestCase {
    private let entries = [("c", ["b"]), ("b", ["a"]), ("a", [])].map { id, parents in
        OpLogEntry(id: ShortId(id: id, shortLen: 1), parentIds: parents, description: id, timestampMillis: 0, isCurrent: id == "c")
    }

    func testConcurrentOperationsDoNotTreatTheNextRowAsAParent() {
        let concurrent = [
            OpLogEntry(id: ShortId(id: "merge", shortLen: 1), parentIds: ["b", "other"], description: "", timestampMillis: 0, isCurrent: true),
            entries[1],
            OpLogEntry(id: ShortId(id: "other", shortLen: 1), parentIds: ["a"], description: "", timestampMillis: 0, isCurrent: false),
            entries[2]
        ]
        XCTAssertNil(concurrent.restoreTarget(undoing: nil))
        XCTAssertEqual(concurrent.restoreTarget(undoing: "b"), "a")
    }

    func testRestoreTargetIsTheOperationBeforeTheOneBeingUndone() {
        XCTAssertEqual(entries.restoreTarget(undoing: nil), "b")
        XCTAssertEqual(entries.restoreTarget(undoing: "b"), "a")
        XCTAssertNil(entries.restoreTarget(undoing: "a"))
        XCTAssertNil(entries.restoreTarget(undoing: "gone"))
    }
}
