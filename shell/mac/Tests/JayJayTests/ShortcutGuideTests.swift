@testable import JayJay
import XCTest

final class ShortcutGuideTests: XCTestCase {
    func testColumnsSplitCoversEverySectionExactlyOnce() {
        let columns = ShortcutGuide.columns
        XCTAssertEqual(columns.count, 2)

        let flattenedTitles = columns.flatMap { $0 }.map(\.title)
        XCTAssertEqual(
            flattenedTitles.sorted(),
            ShortcutGuide.sections.map(\.title).sorted(),
            "every section must appear exactly once across the two columns"
        )
    }

    func testColumnsAreNonEmptyAndBalanced() {
        let columns = ShortcutGuide.columns
        let counts = columns.map { column in column.reduce(0) { $0 + $1.entries.count } }
        XCTAssertTrue(counts.allSatisfy { $0 > 0 }, "neither column should be empty")

        // The greedy split is balanced to within the largest section's entry count.
        let largestSection = ShortcutGuide.sections.map(\.entries.count).max() ?? 0
        XCTAssertLessThanOrEqual(abs(counts[0] - counts[1]), largestSection)
    }
}
