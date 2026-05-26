import AppKit
@testable import JayJayDiffUI
import XCTest

final class SearchRangeTests: XCTestCase {
    func testLastRangeFromLengthReturnsNil() {
        let text = "alpha beta" as NSString

        XCTAssertNil(text.lastRange(of: "alpha", from: text.length))
    }

    func testLastRangeFromStartSearchesSuffixOnly() throws {
        let text = "alpha beta alpha" as NSString

        let range = try XCTUnwrap(text.lastRange(of: "alpha", from: 6))

        XCTAssertEqual(range.location, 11)
    }

    func testFindMatchRangesIgnoresInvalidVisibleGlyphRange() {
        let container = NSTextContainer(containerSize: NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        ))
        let manager = DiffLayoutManager()
        manager.addTextContainer(container)

        let storage = NSTextStorage(string: "trim_output\n")
        storage.addLayoutManager(manager)
        manager.ensureLayout(for: container)

        let ranges = manager.findMatchRanges(
            "trim_output",
            visibleGlyphRange: NSRange(location: NSNotFound, length: 1)
        )

        XCTAssertTrue(ranges.isEmpty)
    }
}
