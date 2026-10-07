@testable import JayJay
import JayJayCore
import XCTest

final class ErrorMessagesTests: XCTestCase {
    func testFriendlyDescriptionUnwrapsCommandErrors() {
        let error = JayError.Internal(message: "command failed: Error: No conflicts found at the given path(s)")
        XCTAssertEqual(error.friendlyDescription, "No conflicts found at the given path(s)")
    }
}
