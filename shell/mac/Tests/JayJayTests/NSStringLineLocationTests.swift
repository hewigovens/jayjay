@testable import JayJay
import XCTest

final class NSStringLineLocationTests: XCTestCase {
    func testLineStartsCountOnlyLineFeedsAndClampToTheEnd() {
        XCTAssertEqual(("a\rb\nc\n" as NSString).location(ofLine: 2), 4)
        let text = "🐦\r\nb\u{2028}c\nd" as NSString
        XCTAssertEqual([0, 1, 2, 3, 9].map { text.location(ofLine: $0) }, [0, 0, 4, 8, 9])
    }
}
