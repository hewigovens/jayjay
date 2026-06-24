@testable import JayJay
import XCTest

final class OperationIconTests: XCTestCase {
    private func symbol(_ description: String) -> String {
        OperationIcon.symbol(for: description)
    }

    func testSpecificVerbsWinOverGenericCommitFallback() {
        // Most op descriptions contain "commit"; the specific verb must match first.
        XCTAssertEqual(symbol("rebase commit abc"), "arrow.triangle.branch")
        XCTAssertEqual(symbol("abandon commit abc"), "trash")
        XCTAssertEqual(symbol("describe commit abc"), "text.bubble")
        XCTAssertEqual(symbol("new empty commit"), "plus.circle")
        XCTAssertEqual(symbol("check out commit abc"), "pencil.circle")
        XCTAssertEqual(symbol("point bookmark main to commit abc"), "bookmark")
    }

    func testSnapshotAndPlainCommitAndUnknown() {
        XCTAssertEqual(symbol("snapshot working copy"), "camera")
        XCTAssertEqual(symbol("commit working copy"), "checkmark.circle")
        XCTAssertEqual(symbol("something unrecognized"), "clock.arrow.circlepath")
    }
}
