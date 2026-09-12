@testable import JayJay
import XCTest

final class StackedPrNamerTests: XCTestCase {
    func testSlugIsNilWhenTheReplyHasNoWords() {
        XCTAssertEqual(StackedPrNamer.slug("\"feat: editable names!\""), "feat-editable-names")
        XCTAssertNil(StackedPrNamer.slug("***"))
    }
}
