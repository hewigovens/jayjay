@testable import JayJay
import XCTest

final class StackedPrNamerTests: XCTestCase {
    func testSlugSanitizesModelOutput() {
        XCTAssertEqual(StackedPrNamer.slug("Add GitLab support"), "add-gitlab-support")
        // Models often wrap the answer in quotes/backticks or add punctuation.
        XCTAssertEqual(StackedPrNamer.slug("\"feat: editable names!\""), "feat-editable-names")
        XCTAssertEqual(StackedPrNamer.slug("`branch-name`"), "branch-name")
        XCTAssertEqual(StackedPrNamer.slug("--leading and trailing--"), "leading-and-trailing")
    }

    func testSlugCapsAtFiveWords() {
        XCTAssertEqual(
            StackedPrNamer.slug("one two three four five six seven"),
            "one-two-three-four-five"
        )
    }

    func testSlugReturnsNilWhenEmpty() {
        XCTAssertNil(StackedPrNamer.slug(""))
        XCTAssertNil(StackedPrNamer.slug("   \n  "))
        XCTAssertNil(StackedPrNamer.slug("***"))
    }
}
