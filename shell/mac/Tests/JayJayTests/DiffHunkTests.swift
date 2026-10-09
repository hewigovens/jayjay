@testable import JayJay
import JayJayCore
import XCTest

final class DiffHunkTests: XCTestCase {
    private func hunk(type: HunkType, matchesSource: Bool) -> DiffHunk {
        testHunk(path: "new/path", oldPath: "old/path", hunkType: type, reviewIdentity: "", matchesSource: matchesSource)
    }

    func testOnlyVerifiedMatchesAreContentFree() {
        // Summary hunks carry no content, so an edited copy or rename must still load its diff.
        XCTAssertFalse(hunk(type: .copied, matchesSource: false).isContentFreeRenameOrCopy)
        XCTAssertFalse(hunk(type: .renamed, matchesSource: false).isContentFreeRenameOrCopy)
        XCTAssertTrue(hunk(type: .copied, matchesSource: true).isContentFreeRenameOrCopy)
        XCTAssertTrue(hunk(type: .renamed, matchesSource: true).isContentFreeRenameOrCopy)
    }

    func testFileActionsCarryARenameSourceButNotACopySource() {
        XCTAssertEqual(hunk(type: .renamed, matchesSource: false).changedPaths, ["new/path", "old/path"])
        XCTAssertEqual(hunk(type: .copied, matchesSource: false).changedPaths, ["new/path"])
    }
}
