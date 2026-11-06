@testable import JayJay
import JayJayCore
import XCTest

final class ShortcutGuideTests: XCTestCase {
    func testColumnsSplitCoversEverySectionExactlyOnce() {
        let bindings = shortcutBindings(stored: [:]).filter { !$0.chords.isEmpty }
        let columns = ShortcutGuide.columns(bindings: bindings)
        XCTAssertEqual(columns.count, 2)

        let flattenedTitles = columns.flatMap { $0 }.map(\.title)
        XCTAssertEqual(
            flattenedTitles.sorted(),
            ShortcutGuide.sections(bindings: bindings).map(\.title).sorted(),
            "every section must appear exactly once across the two columns"
        )
    }

    func testColumnsAreNonEmptyAndBalanced() {
        let bindings = shortcutBindings(stored: [:]).filter { !$0.chords.isEmpty }
        let columns = ShortcutGuide.columns(bindings: bindings)
        let counts = columns.map { column in column.reduce(0) { $0 + $1.entries.count } }
        XCTAssertTrue(counts.allSatisfy { $0 > 0 }, "neither column should be empty")

        let largestSection = ShortcutGuide.sections(bindings: bindings).map(\.entries.count).max() ?? 0
        XCTAssertLessThanOrEqual(abs(counts[0] - counts[1]), largestSection)
    }
}
