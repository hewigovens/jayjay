@testable import JayJay
import XCTest

final class DAGNodeStyleTests: XCTestCase {
    func testImmutableChangesDrawAsDiamondsLikeJjLog() {
        XCTAssertEqual(DAGNodeStyle.resolve(change: mockChangeInfo(isImmutable: true)).shape, .diamond)
        XCTAssertEqual(DAGNodeStyle.resolve(change: mockChangeInfo(isImmutable: false)).shape, .circle)
    }

    func testMutableChangeWithTrunkBookmarkStaysACircle() {
        XCTAssertEqual(DAGNodeStyle.resolve(change: mockChangeInfo(bookmarks: ["main"])).shape, .circle)
    }
}
