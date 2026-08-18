@testable import JayJay
import XCTest

final class TrunkBookmarksTests: XCTestCase {
    func testChipCanRemoveAConflictedTrunkTarget() {
        XCTAssertTrue(canRemoveBookmarkFromChip("main", conflicted: true))
        XCTAssertTrue(canRemoveBookmarkFromChip("main@origin", conflicted: true))
        XCTAssertTrue(canRemoveBookmarkFromChip("feature", conflicted: false))
        XCTAssertFalse(canRemoveBookmarkFromChip("main", conflicted: false))
        XCTAssertFalse(canRemoveBookmarkFromChip("master", conflicted: false))
    }
}
