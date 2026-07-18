@testable import JayJay
import XCTest

final class PendingPushConfirmationTests: XCTestCase {
    func testRejectedPushPreservesPendingBookmark() {
        let remaining = PendingPushConfirmation.remainingBookmark(
            afterConfirming: "main",
            startPush: { _ in false }
        )

        XCTAssertEqual(remaining, "main")
    }

    func testAcceptedPushClearsPendingBookmark() {
        let remaining = PendingPushConfirmation.remainingBookmark(
            afterConfirming: "main",
            startPush: { _ in true }
        )

        XCTAssertNil(remaining)
    }
}
